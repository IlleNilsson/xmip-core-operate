using Xmip.Abi.Operate;
using Xmip.Operate;

namespace Xmip.Surface;

/// <summary>
/// The surface over a published snapshot: what a node — or the System Tests,
/// after every tick — wrote to a file, read once per publication into a
/// <see cref="ScopeIndex"/> and answered from that by lookup. The path is
/// given by whoever chose this surface (ADR-0052 clause 3); nothing here
/// guesses at a temp directory, and a path with no file behind it says so in
/// <see cref="Source"/> rather than reporting an empty estate as if it were one.
/// </summary>
/// <remarks>
/// The surface is the runtime's, <c>operate::surface::snapshot</c>, opened
/// across the C ABI (<c>xmip_operate.h</c> section 28.2): when the file is
/// read again, what it reads as — <c>observe::Publication</c>'s one reader —
/// its scope tree and what a follow tells are its, the one the command line
/// reads in Rust (ADR-0052, amendment 2026-11-18). A read-only surface: it
/// reports what was published and never acts on it, so
/// <see cref="PauseScope"/>, <see cref="ResumeScope"/> and every act decline,
/// in the runtime's words; an act is asked at a node's door (ADR-0014,
/// amendment 2026-11-13). ADR-0027, ADR-0028.
///
/// A followed surface is told, it does not ask (ADR-0052, amendment
/// 2026-09-15): while anything follows <see cref="WatchAsync"/>, the
/// runtime's follow reads each publication once as the operating system
/// tells it the file changed, and every question is answered from that
/// reading without touching the file. Until 2026-10-03 the page rendered
/// after each System Tests tick read the 2 MB snapshot itself (867 ms,
/// measured 2026-10-02).
/// </remarks>
public sealed class SnapshotOperator(string path) : IOperatorSurface, IDisposable
{
    private readonly HeldSurface held =
        new(RuntimeLibrary.Rules.Surfaces.Snapshot(path), readsAhead: true);

    /// <summary>Why a snapshot takes no act, in the runtime's words.</summary>
    public static string NotActed => RuntimeLibrary.Rules.Surfaces.NotActed;

    /// <summary>The file this surface reads.</summary>
    public string Path { get; } = path;

    /// <summary>Whether there is a file to read.</summary>
    public bool Exists => held.Node.Answers();

    /// <inheritdoc />
    public string Source => held.Source;

    /// <inheritdoc />
    public ScopeIndex Index()
    {
        return held.Read().Index;
    }

    /// <inheritdoc />
    public IReadOnlyList<HealthRecord> Health(string scope)
    {
        return held.Read().Index.Health(scope);
    }

    /// <inheritdoc />
    public Figures Figures(string scope)
    {
        return held.Read().Index.Figures(scope);
    }

    /// <inheritdoc />
    public MeasurementRecord? Measure(string scope, Counted counted)
    {
        return held.Read().Index.Measure(scope, counted);
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
    public IAsyncEnumerable<SurfaceChange> WatchAsync(CancellationToken stop = default)
    {
        return held.WatchAsync(stop);
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
            caller, subscription.Node, "event-subscription",
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
        return held.Control(scope, ScopeAction.Pause, who).Result;
    }

    /// <inheritdoc />
    public string ResumeScope(string scope, HeldCaller? caller)
    {
        return held.Control(scope, ScopeAction.Resume, string.Empty).Result;
    }

    /// <inheritdoc />
    public ScopeOperation Control(string scope, ScopeAction action, string who, HeldCaller? caller)
    {
        return held.Control(scope, action, who);
    }

    /// <summary>Release the surface and end its follow.</summary>
    public void Dispose()
    {
        held.Dispose();
    }
}
