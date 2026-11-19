namespace Xmip.Operate;

// Each section of the operating header a surface asks, as the runtime's rules
// hold it: bound at load where every build exports it, else where it is
// first asked and a remote build refused in words (RemoteBuild).
public sealed partial class RuntimeRules
{
    /// <summary>Section 8: a publication read by the runtime's one
    /// reader.</summary>
    public PublicationReader Publications { get; }

    /// <summary>Section 9: a program's audit record, recorded by the audit
    /// capability (ADR-0062).</summary>
    public RuntimeAudit Audit { get; }

    /// <summary>Section 10: the cluster's <c>cluster.xmip</c> read, edited and
    /// sliced (ADR-0064, amendment 2026-10-03; ADR-0031, amendment
    /// 2026-10-05).</summary>
    public RuntimeDesign Design =>
        field ?? throw RemoteBuild.Refused(_library, RuntimeDesign.Entrypoints);

    /// <summary>Section 11: Events, subscribed from .NET and published into
    /// this process (ADR-0065).</summary>
    public RuntimeEvents Events =>
        field ?? throw RemoteBuild.Refused(_library, RuntimeEvents.Entrypoints);

    /// <summary>Section 12: the technologies the runtime carries and the
    /// settings each declares (ADR-0064, amendment 2026-09-26).</summary>
    public RuntimeCatalogue Catalogue =>
        field ?? throw RemoteBuild.Refused(_library, RuntimeCatalogue.Entrypoints);

    /// <summary>Section 13: a System Process declared, and the declarations
    /// that stand (ADR-0053 clause 3).</summary>
    public RuntimeProcesses Processes =>
        field ?? throw RemoteBuild.Refused(_library, RuntimeProcesses.Entrypoints);

    /// <summary>Section 11, what an operator lists and does: the Event
    /// subscriptions a hub holds, and pause, resume and remove on one
    /// (ADR-0065, amendment 2026-09-29).</summary>
    public RuntimeEventSubscriptions EventSubscriptions =>
        field ?? throw RemoteBuild.Refused(_library, RuntimeEventSubscriptions.Entrypoints);

    /// <summary>Section 14: the Subscriptions of the nodes running in this
    /// process, pause and resume on one, and an operator's order on any
    /// noun (ADR-0013, amendment 2026-09-30).</summary>
    public RuntimeSubscriptions Subscriptions =>
        field ?? throw RemoteBuild.Refused(_library, RuntimeSubscriptions.Entrypoints);

    /// <summary>Section 15: the Dead Message Queues of the nodes running in
    /// this process, and Replay on one of their Messages (ADR-0052,
    /// amendment 2026-10-01).</summary>
    public RuntimeDeadMessages DeadMessages =>
        field ?? throw RemoteBuild.Refused(_library, RuntimeDeadMessages.Entrypoints);

    /// <summary>Section 16: Retry and Dismiss on a Message that failed, sent
    /// by a node running in this process (runtime-model.md section 13;
    /// ADR-0013).</summary>
    public RuntimeMessages Messages =>
        field ?? throw RemoteBuild.Refused(_library, RuntimeMessages.Entrypoints);

    /// <summary>Section 17: a node reached at its operate address over gRPC
    /// inside Xmip's mutual TLS, and this process's node served on an operate
    /// listener of its own (ADR-0014, amendment 2026-10-12).</summary>
    public RuntimeRemotes Remotes { get; }

    /// <summary>Section 18: a person's security role and the one check every
    /// act passes (ADR-0009).</summary>
    public RuntimeSecurityRoles SecurityRoles { get; }

    /// <summary>Section 19: a person's key and request made, and signed on a
    /// node's build by its authority (ADR-0034, amendment 2026-10-15).</summary>
    public RuntimeCertificates Certificates { get; }

    /// <summary>Section 20: the scripts that upgrade each database on a server
    /// from one schema version to a later one (ADR-0015, amendment
    /// 2026-10-20). A node's build only.</summary>
    public RuntimeDatabases Databases =>
        field ?? throw RemoteBuild.Refused(_library, RuntimeDatabases.Entrypoints);

    /// <summary>Section 21: a web host's HTTPS in Xmip's own TLS, and its
    /// certificate under an authority kept in a folder (ADR-0009, amendment
    /// 2026-10-21). A node's build only.</summary>
    public RuntimeSecureSurface Hosting =>
        field ?? throw RemoteBuild.Refused(_library, RuntimeSecureSurface.Entrypoints);

    /// <summary>Section 25: every Report listed, each saying whether it is
    /// quick or a Quest probes first, and one opened (ADR-0072, amendment
    /// 2026-11-10). A node's build only.</summary>
    public RuntimeReports Reports =>
        field ?? throw RemoteBuild.Refused(_library, RuntimeReports.Entrypoints);

    /// <summary>Section 26: a module library probed through the runtime's one
    /// loader and judged by the rule a node judges it by (ADR-0012, amendment
    /// 2026-11-17). A node's build only.</summary>
    public RuntimeModules Modules =>
        field ?? throw RemoteBuild.Refused(_library, RuntimeModules.Entrypoints);

    /// <summary>Section 27: a configuration document read, written and one
    /// key of it set by the runtime's one reader (ADR-0031, amendment
    /// 2026-11-17).</summary>
    public RuntimeDocuments Documents { get; }

    /// <summary>Section 28.1: a publication's scope tree, built once and
    /// asked by scope (ADR-0052, amendment 2026-11-18).</summary>
    public RuntimeScopeIndexes ScopeIndexes { get; }

    /// <summary>Section 28.2: a surface opened, read, followed and asked to
    /// act (ADR-0052, amendment 2026-11-18).</summary>
    public RuntimeSurfaces Surfaces { get; }

    /// <summary>Section 28.3: what a surface says and how it orders a list
    /// (ADR-0052, amendment 2026-11-18).</summary>
    public RuntimeWords Sentences { get; }
}
