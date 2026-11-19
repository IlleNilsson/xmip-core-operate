use abi::operate::scope_index::{self as header, Figures, ScopeIndexFacts, list, relative};
use abi::operate::{HealthEntry, Measurement, counted, health};
use observe::{Health, HealthRecord, Publication, Snapshot};

use super::asked::{
    xmip_scope_index_label_v1, xmip_scope_index_list_v1, xmip_scope_index_measure_v1,
    xmip_scope_index_select_v1, xmip_scope_index_worst_at_stage_v1,
};
use super::tree::{xmip_figures_sum_v1, xmip_scope_relative_v1};
use super::*;
use crate::ffi::publication::{xmip_publication_free_v1, xmip_publication_read_v1};

#[test]
fn the_exports_have_the_shapes_the_header_declares() {
    let _: header::ScopeIndexBuildFn = xmip_scope_index_build_v1;
    let _: header::ScopeIndexOfPublicationFn = xmip_scope_index_of_publication_v1;
    let _: header::ScopeIndexFreeFn = xmip_scope_index_free_v1;
    let _: header::ScopeIndexFactsFn = xmip_scope_index_facts_v1;
    let _: header::ScopeIndexPlacedFn = xmip_scope_index_placed_v1;
    let _: header::ScopeIndexChildrenFn = xmip_scope_index_children_v1;
    let _: header::ScopeIndexListFn = xmip_scope_index_list_v1;
    let _: header::ScopeIndexLabelFn = xmip_scope_index_label_v1;
    let _: header::ScopeIndexWorstAtStageFn = xmip_scope_index_worst_at_stage_v1;
    let _: header::ScopeIndexMeasureFn = xmip_scope_index_measure_v1;
    let _: header::ScopeIndexCapabilitiesFn = super::asked::xmip_scope_index_capabilities_v1;
    let _: header::ScopeIndexSelectFn = xmip_scope_index_select_v1;
    let _: header::ScopeRelativeFn = xmip_scope_relative_v1;
    let _: header::FiguresSumFn = xmip_figures_sum_v1;
}

// The test cluster's first node, its receive Location and its Send Location.
fn node() -> String {
    configure::fixture::test_cluster().node_scope(0)
}

fn received() -> String {
    format!("{}/receive/tcp", node())
}

fn sent() -> String {
    format!("{}/send/out", node())
}

fn text(value: Str) -> String {
    if value.ptr.is_null() {
        return String::new();
    }
    // SAFETY: the export promises `len` readable bytes.
    let bytes = unsafe { core::slice::from_raw_parts(value.ptr, value.len) };
    String::from_utf8(bytes.to_vec()).expect("UTF-8")
}

// An index of two records — fine at the receive Location, holding at the
// Send Location, published in that order — and one count beneath the node.
fn built() -> *mut Handle {
    let (received, sent) = (received(), sent());
    let records = [
        HealthEntry {
            scope: borrow(&received),
            health: health::FINE,
            severity: 0,
            evidence: borrow(""),
            observed_unix_nanos: 5,
        },
        HealthEntry {
            scope: borrow(&sent),
            health: health::HOLDING,
            severity: 70,
            evidence: borrow("disk full"),
            observed_unix_nanos: 6,
        },
    ];
    let counts = [Measurement {
        scope: borrow(&received),
        counted: counted::MESSAGES,
        value: 3,
        window_start_unix_nanos: 1,
        window_end_unix_nanos: 9,
        observed_unix_nanos: 9,
    }];
    let mut index = core::ptr::null_mut();
    // SAFETY: two records, one count, each string borrowed for the call.
    let code = unsafe {
        xmip_scope_index_build_v1(
            records.as_ptr(),
            2,
            counts.as_ptr(),
            1,
            4,
            borrow("t"),
            &raw mut index,
        )
    };
    assert_eq!(code, status::OK);
    index
}

#[test]
fn a_built_index_answers_a_scope_by_its_records_places() {
    let index = built();
    let at = node();
    let mut node = core::mem::MaybeUninit::<ScopePlaced>::uninit();
    // SAFETY: a live handle; `node` is writable.
    let code =
        unsafe { xmip_scope_index_placed_v1(index, borrow(&format!("{at}/")), node.as_mut_ptr()) };
    assert_eq!(code, status::OK);
    // SAFETY: OK wrote it.
    let node = unsafe { node.assume_init() };
    // SAFETY: the places are the index's, `leaves_len` of them.
    let leaves = unsafe { core::slice::from_raw_parts(node.leaves, node.leaves_len) };

    assert_eq!(text(node.scope), at);
    assert_eq!(
        leaves,
        [1, 0],
        "worst first, by the place each was built at"
    );
    assert_eq!((node.own, node.is_container, node.is_leaf), (-1, 1, 0));
    assert_eq!(node.row_mood, wire_health(Health::Holding.rolled()));
    assert_eq!(
        (node.published, node.messages),
        (header::figure::MESSAGES, 3)
    );

    let mut children = [core::mem::MaybeUninit::<ScopePlaced>::uninit(); 4];
    let mut count = 0;
    // SAFETY: a live handle; room for four entries.
    let code = unsafe {
        xmip_scope_index_children_v1(
            index,
            borrow(&at),
            children[0].as_mut_ptr(),
            4,
            &raw mut count,
        )
    };
    assert_eq!((code, count), (status::OK, 2));
    // SAFETY: two were written.
    let first = unsafe { children[0].assume_init() };
    assert_eq!(text(first.label), "send", "the worst first");
    assert_eq!(text(first.stage), "send");
    assert_eq!(first.begins_stage, 1);

    let test = configure::fixture::test_cluster();
    let absent = format!("{}/node/{}", test.scope(), test.absent());
    let mut none = core::mem::MaybeUninit::<ScopePlaced>::uninit();
    // SAFETY: as above.
    let missing = unsafe { xmip_scope_index_placed_v1(index, borrow(&absent), none.as_mut_ptr()) };
    assert_eq!(missing, status::NOT_FOUND);

    let mut facts = core::mem::MaybeUninit::<ScopeIndexFacts>::uninit();
    // SAFETY: a live handle; `facts` is writable.
    assert_eq!(
        unsafe { xmip_scope_index_facts_v1(index, facts.as_mut_ptr()) },
        status::OK
    );
    // SAFETY: OK wrote it.
    let facts = unsafe { facts.assume_init() };
    assert_eq!(
        (
            facts.revision,
            facts.leaves,
            facts.observed,
            facts.observed_unix_nanos
        ),
        (4, 2, 1, 9)
    );
    assert_eq!(text(facts.source), "t");

    // SAFETY: freed once, nothing borrowed used after.
    unsafe { xmip_scope_index_free_v1(index) };
}

#[test]
fn an_index_lists_labels_measures_and_selects() {
    let index = built();
    let (at, sent) = (node(), sent());
    let cluster = configure::fixture::test_cluster().scope();
    let mut listed = [Str::empty(); 4];
    let mut count = 0;
    // SAFETY: a live handle; room for four.
    let code = unsafe {
        xmip_scope_index_list_v1(
            index,
            list::STAGE_SCOPES,
            borrow("send"),
            listed.as_mut_ptr(),
            4,
            &raw mut count,
        )
    };
    assert_eq!((code, count), (status::OK, 1));
    assert_eq!(text(listed[0]), format!("{at}/send"));
    let pattern = format!("{cluster}/node/*/send/*");
    // SAFETY: as above.
    let matching = unsafe {
        xmip_scope_index_list_v1(
            index,
            list::MATCHING,
            borrow(&pattern),
            listed.as_mut_ptr(),
            4,
            &raw mut count,
        )
    };
    assert_eq!((matching, count, text(listed[0])), (status::OK, 1, sent));

    let stage = format!("{at}/send");
    let mut label = Str::empty();
    // SAFETY: a live handle; `label` is writable.
    assert_eq!(
        unsafe { xmip_scope_index_label_v1(index, borrow(&stage), &raw mut label) },
        status::OK
    );
    assert_eq!(text(label), "Send");

    let mut place = 0;
    // SAFETY: as above.
    assert_eq!(
        unsafe { xmip_scope_index_worst_at_stage_v1(index, borrow("send"), &raw mut place) },
        status::OK
    );
    assert_eq!(place, 1);

    let received = received();
    let mut measured = core::mem::MaybeUninit::<Measurement>::uninit();
    // SAFETY: as above.
    let code = unsafe {
        xmip_scope_index_measure_v1(
            index,
            borrow(&received),
            counted::MESSAGES,
            100,
            measured.as_mut_ptr(),
        )
    };
    assert_eq!(code, status::OK);
    // SAFETY: OK wrote it.
    let measured = unsafe { measured.assume_init() };
    assert_eq!((measured.value, measured.observed_unix_nanos), (3, 9));

    let nothing = format!("{cluster}/nothing*");
    let mut out = vec![0u8; 512];
    let mut len = 0;
    // SAFETY: room for 512 bytes.
    let refused = unsafe {
        xmip_scope_index_select_v1(
            index,
            borrow(&nothing),
            borrow("SRC"),
            out.as_mut_ptr(),
            out.len(),
            &raw mut len,
        )
    };
    assert_eq!(refused, status::NOT_FOUND);
    let said = String::from_utf8(out[..len].to_vec()).expect("UTF-8");
    assert!(
        said.starts_with("REFUSED  no scope matches") && said.ends_with("source SRC"),
        "{said}"
    );

    // SAFETY: freed once.
    unsafe { xmip_scope_index_free_v1(index) };
}

#[test]
fn an_index_of_a_publication_keeps_the_order_its_records_are_handed_out() {
    let sent = sent();
    let mut snapshot = Snapshot::new();
    for (scope, health, severity) in [
        (received(), Health::Fine, 0),
        (sent.clone(), Health::Stressed, 40),
    ] {
        snapshot.record_health(HealthRecord {
            scope,
            health,
            severity,
            evidence: String::new(),
            observed_unix_nanos: 1,
        });
    }
    let cluster = configure::fixture::test_cluster().scope();
    let published = Publication::whole("p", &cluster, &snapshot).to_toml();
    let (mut read, mut index) = (core::ptr::null_mut(), core::ptr::null_mut());
    let mut len = 0;
    // SAFETY: the text is borrowed for the call; `read` and `len` writable.
    let code = unsafe {
        xmip_publication_read_v1(
            borrow(&published),
            &raw mut read,
            core::ptr::null_mut(),
            0,
            &raw mut len,
        )
    };
    assert_eq!(code, status::OK);
    // SAFETY: a live publication; `index` is writable.
    assert_eq!(
        unsafe { xmip_scope_index_of_publication_v1(read, 2, borrow("p"), &raw mut index) },
        status::OK
    );
    // SAFETY: freed once; the index holds what it needs.
    unsafe { xmip_publication_free_v1(read) };

    let held = unsafe { held(index) }.expect("an index");
    let worst = held
        .placed(&sent)
        .and_then(|placed| placed.leaves.first().copied());
    assert_eq!(
        held.records()[worst.expect("a record") as usize].scope,
        sent
    );
    // SAFETY: freed once.
    unsafe { xmip_scope_index_free_v1(index) };
}

#[test]
fn a_scope_s_relatives_and_figures_summed_are_the_tree_s() {
    let ask = |scope: &str, which| {
        let mut out = vec![0u8; 256];
        let mut len = 0;
        // SAFETY: room for 256 bytes.
        let code = unsafe {
            xmip_scope_relative_v1(
                borrow(scope),
                which,
                out.as_mut_ptr(),
                out.len(),
                &raw mut len,
            )
        };
        assert_eq!(code, status::OK);
        String::from_utf8(out[..len].to_vec()).expect("UTF-8")
    };
    let test = configure::fixture::test_cluster();
    let (at, sent) = (node(), sent());
    assert_eq!(ask(&sent, relative::PARENT), format!("{at}/send"));
    assert_eq!(ask(&test.scope(), relative::PARENT), "xmip:///");
    assert_eq!(ask(&sent, relative::NODE_SCOPE), at);
    assert_eq!(
        ask(&sent, relative::NAME),
        format!("node/{}/send/out", test.node(0).name)
    );

    let parts = [
        Figures {
            streams: 2,
            published: header::figure::STREAMS,
            observed: 1,
            observed_unix_nanos: 5,
            ..Figures::default()
        },
        Figures {
            streams: 3,
            failed: 1,
            published: header::figure::STREAMS | header::figure::FAILED,
            ..Figures::default()
        },
    ];
    let mut sum = Figures::default();
    // SAFETY: two parts; `sum` is writable.
    assert_eq!(
        unsafe { xmip_figures_sum_v1(parts.as_ptr(), 2, &raw mut sum) },
        status::OK
    );
    assert_eq!(
        (sum.streams, sum.failed, sum.observed_unix_nanos),
        (5, 1, 5)
    );
    assert_eq!(
        sum.published,
        header::figure::STREAMS | header::figure::FAILED
    );
}
