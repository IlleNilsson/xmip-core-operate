using Xmip.Abi.Operate;
using Xmip.Operate;

namespace Xmip.Surface;

/// <summary>
/// The surface over a node's operate listener, followed over gRPC (ADR-0014,
/// amendments 2026-10-12 and 2026-10-13; ADR-0052, amendment 2026-10-05),
/// which the command line, the PowerShell module and the GUIs all operate a
/// node through — <see cref="LocalOperator"/> on this machine, at the node's
/// local endpoint, and <see cref="RemoteOperator"/> on another, at its
/// operate address inside Xmip's mutual TLS. SignalR stays between a web
/// host and its browser.
/// </summary>
/// <remarks>
/// The surface is the runtime's, <c>operate::surface::listener</c>, opened
/// across the C ABI (<c>xmip_operate.h</c> section 28.2), the one the command
/// line reads in Rust (ADR-0052, amendment 2026-11-18). It follows what the
/// runtime calls answered: the cluster's view where the node it reaches
/// gathers one — a node declaring the monitoring role — and the node's own
/// publication where the node refuses, its answer deciding as each
/// connection to it is made, never by a name (the design approved
/// 2026-11-12, section 3.4). It reads what it follows once per revision and
/// is told, never asks, when it advances; a lost node is told too, every
/// answer empty until it is back — never the publication held from before —
/// and the runtime's follow tries it again on its own loop.
///
/// An act crosses without a name: the node takes it as the identity the
/// connection proved — this side's certificate, or on the local endpoint
/// the user this process runs as — admitted by the role the node grants
/// it, and refuses it in words otherwise (ADR-0009). The <c>who</c> an act
/// is given here does not leave this process; a <c>caller</c> does: the node
/// is handed it and decides by its own role (ADR-0009, amendment 2026-11-01).
/// </remarks>
public abstract partial class ListenerOperator : IOperatorSurface, IDisposable
{
    private readonly HeldSurface held;

    /// <summary>A surface over the node the runtime opened as
    /// <paramref name="node"/>, called <paramref name="address"/> in what it
    /// says, of the <paramref name="kind"/> its <see cref="Source"/> opens
    /// with.</summary>
    private protected ListenerOperator(string kind, string address, SurfaceNode node)
    {
        Kind = kind;
        Address = address;
        held = new HeldSurface(node, readsAhead: false);
    }

    /// <summary>Where the node is, as <see cref="Source"/> opens: <c>LOCAL</c>
    /// on this machine, <c>REMOTE</c> on another.</summary>
    public string Kind { get; }

    /// <summary>Where the node is reached: its operate address, or its local
    /// endpoint.</summary>
    public string Address { get; }

    /// <summary>Whether the node answered when last asked or told.</summary>
    public bool IsConnected => held.Node.Standing().Connected;

    /// <summary>Why the node does not answer, when it does not.</summary>
    public string Reason => held.Node.Standing().Reason;

    /// <summary>Whether the node this reaches gathers its cluster's view, so
    /// that this follows that view and answers for the whole cluster: false
    /// until the node's answer decides, while it is lost, and where it
    /// gathers none.</summary>
    public bool Gathers => held.Node.Standing().Gathers;

    /// <inheritdoc />
    /// <remarks>Where the node gathers its cluster's view, what that view says
    /// of itself follows — <c>&lt;cluster&gt;, gathered by &lt;node&gt;</c> and
    /// how many it reached, the runtime's words and never this one's.</remarks>
    public string Source => held.Source;

    /// <summary>Ask the node for its publication: whether it answered, why
    /// not in <see cref="Reason"/>.</summary>
    public bool Connect()
    {
        return held.Node.Answers();
    }

    /// <inheritdoc />
    public ScopeIndex Index()
    {
        return held.Read().Index;
    }

    /// <inheritdoc />
    public IReadOnlyList<HealthRecord> Health(string scope)
    {
        return Index().Health(scope);
    }

    /// <inheritdoc />
    public Figures Figures(string scope)
    {
        return Index().Figures(scope);
    }

    /// <inheritdoc />
    public MeasurementRecord? Measure(string scope, Counted counted)
    {
        return Index().Measure(scope, counted);
    }

    /// <inheritdoc />
    public TopologySnapshot Topology()
    {
        return held.Read().Topology;
    }

    /// <inheritdoc />
    public RunHeader Run()
    {
        return held.Read().Run;
    }

    /// <inheritdoc />
    public PublishedNaming Naming()
    {
        return held.Read().Naming;
    }

    /// <inheritdoc />
    public string Root()
    {
        return held.Read().Root;
    }

    /// <inheritdoc />
    public EventSubscriptionList EventSubscriptions()
    {
        return held.Read().EventSubscriptions;
    }

    /// <inheritdoc />
    public EventSubscriptionOperation Act(
        EventSubscriptionRecord subscription, EventSubscriptionAct act, string who,
        HeldCaller? caller)
    {
        ArgumentNullException.ThrowIfNull(subscription);

        (ActOutcome outcome, string said) = held.Act(
            caller,
            subscription.Node, "event-subscription",
            subscription.Id.ToString(System.Globalization.CultureInfo.InvariantCulture),
            EventSubscriptionOperation.Word(act));

        return new EventSubscriptionOperation(
            subscription.Node, subscription.Id, act, outcome == ActOutcome.Taken, said)
        {
            Outcome = outcome,
        };
    }

    /// <inheritdoc />
    public SubscriptionList Subscriptions()
    {
        return held.Read().Subscriptions;
    }

    /// <inheritdoc />
    public SubscriptionOperation Act(
        SubscriptionRecord subscription, SubscriptionAct act, string who, HeldCaller? caller)
    {
        ArgumentNullException.ThrowIfNull(subscription);

        (ActOutcome outcome, string said) = held.Act(
            caller, subscription.Node, "subscription", subscription.Name,
            SubscriptionOperation.Word(act));

        return new SubscriptionOperation(
            subscription.Node, subscription.Name, act, outcome == ActOutcome.Taken, said)
        {
            Outcome = outcome,
        };
    }

    /// <inheritdoc />
    public DeadMessageList DeadMessages()
    {
        return held.Read().DeadMessages;
    }

    /// <inheritdoc />
    public WorkProcessInstanceList WorkProcessInstances()
    {
        return held.Read().WorkProcessInstances;
    }

    /// <inheritdoc />
    public DeadMessageOperation Act(
        DeadMessageRecord message, DeadMessageAct act, string who, HeldCaller? caller)
    {
        ArgumentNullException.ThrowIfNull(message);

        (ActOutcome outcome, string said) = held.Act(
            caller, message.Node, DeadMessageOperation.Noun, message.Message,
            DeadMessageOperation.Word(act));

        return new DeadMessageOperation(
            message.Node, message.Message, act, outcome == ActOutcome.Taken, said)
        {
            Outcome = outcome,
        };
    }

    /// <inheritdoc />
    /// <remarks>What the node's publication carries: each Port's count and
    /// its window of the oldest, <paramref name="from"/> a place in the
    /// Port's queue and no next (<see cref="IOperatorSurface.FailedMessages"/>
    /// says why).</remarks>
    public FailedMessageList FailedMessages(string scope, ulong from = 0, uint most = 0)
    {
        return held.Failed(scope, from, most);
    }

    /// <inheritdoc />
    public MessageOperation Act(
        string scope, string journey, MessageAct act, string who, HeldCaller? caller)
    {
        return held.ActOnMessage(caller, scope, journey, act);
    }

    /// <inheritdoc />
    public string PauseScope(string scope, string who, HeldCaller? caller)
    {
        return Control(scope, ScopeAction.Pause, who, caller).Result;
    }

    /// <inheritdoc />
    public string ResumeScope(string scope, HeldCaller? caller)
    {
        return Control(scope, ScopeAction.Resume, string.Empty, caller).Result;
    }

    /// <inheritdoc />
    /// <remarks>A node's operate listener takes the acts on what it
    /// publishes, not a pause of a scope: declined in the runtime's
    /// words.</remarks>
    public ScopeOperation Control(
        string scope, ScopeAction action, string who, HeldCaller? caller)
    {
        return held.Control(scope, action, who);
    }

    /// <inheritdoc />
    public IAsyncEnumerable<SurfaceChange> WatchAsync(CancellationToken stop = default)
    {
        return held.WatchAsync(stop);
    }

    /// <inheritdoc />
    public void Dispose()
    {
        Dispose(true);
        GC.SuppressFinalize(this);
    }

    /// <summary>End every watch, the follow and the node's handle.</summary>
    protected virtual void Dispose(bool disposing)
    {
        if (disposing)
        {
            held.Dispose();
        }
    }
}
