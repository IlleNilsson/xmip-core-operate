using System.Text.Json;

namespace Xmip.Operate;

/// <summary>
/// One entry of an Instance's history of Messages (ADR-0066, amendment
/// 2026-11-25): <c>observe::InstanceHandled</c>, read from the JSON section
/// 29 writes.
/// </summary>
/// <param name="At">When.</param>
/// <param name="What">What happened to the Message: <c>started</c>,
/// <c>arrived</c>, <c>due</c>, <c>made</c>, <c>published</c>, <c>failed</c>,
/// <c>retried</c> or <c>dismissed</c>, the runtime's word.</param>
/// <param name="Message">The Message's identifier.</param>
/// <param name="Journey">The Journey of the turn's identifier.</param>
public sealed record InstanceHandledRecord(
    DateTimeOffset At, string What, string Message, string Journey);

/// <summary>
/// One Instance of a Work Process (runtime-model.md section 22; ADR-0066,
/// amendment 2026-11-25): <c>observe::WorkProcessInstance</c>, read from the
/// JSON section 29 writes. <see cref="Node"/>, <see cref="WorkProcess"/> and
/// <see cref="Instance"/> together name it.
/// </summary>
/// <param name="Node">The node that runs its Work Process.</param>
/// <param name="WorkProcess">The Work Process, by its configured name.</param>
/// <param name="Instance">Its identifier: the Journey of the Message that
/// started it.</param>
/// <param name="State"><c>running</c>, <c>waiting</c>, <c>failed</c>,
/// <c>dismissed</c> or <c>completed</c>, the runtime's word.</param>
/// <param name="WaitsFor"><c>message</c>, <c>time</c>, <c>message or
/// time</c>, or empty.</param>
/// <param name="Until">Its earliest deadline, or null.</param>
/// <param name="Correlations">Each Correlation it holds a key in, and the
/// key.</param>
/// <param name="Code">The error no scope caught, empty where it has not
/// failed.</param>
/// <param name="Reason">Why, in words.</param>
/// <param name="FailedBy">The Journey whose turn failed it: what an
/// operator's Retry or Dismiss names; empty where it has not failed.</param>
/// <param name="Values">Its own values, by name.</param>
/// <param name="History">The Messages it handled, oldest first.</param>
/// <param name="Earlier">How many entries of its history came before those
/// kept.</param>
/// <param name="Turns">How many turns it has taken.</param>
/// <param name="Updated">When Xmip Storage last wrote it.</param>
public sealed record WorkProcessInstanceRecord(
    string Node,
    string WorkProcess,
    string Instance,
    string State,
    string WaitsFor,
    DateTimeOffset? Until,
    IReadOnlyList<KeyValuePair<string, string>> Correlations,
    string Code,
    string Reason,
    string FailedBy,
    IReadOnlyList<KeyValuePair<string, string>> Values,
    IReadOnlyList<InstanceHandledRecord> History,
    ulong Earlier,
    ulong Turns,
    DateTimeOffset Updated);

/// <summary>
/// One Work Process's Instances on one node, and the Messages that failed in
/// its queue: <c>observe::WorkProcessInstances</c>. A failed Message is a
/// <see cref="FailedMessageRecord"/> whose port is empty: it waits at the Work
/// Process, and takes the acts every failed Message takes.
/// </summary>
/// <param name="Node">The node that runs it.</param>
/// <param name="WorkProcess">The Work Process, by its configured name.</param>
/// <param name="FailedCount">How many Messages that failed wait in its queue
/// now.</param>
/// <param name="Failed">The oldest of them, oldest first.</param>
/// <param name="Instances">Its Instances, newest written first.</param>
public sealed record WorkProcessInstances(
    string Node,
    string WorkProcess,
    ulong FailedCount,
    IReadOnlyList<FailedMessageRecord> Failed,
    IReadOnlyList<WorkProcessInstanceRecord> Instances);

/// <summary>
/// The Instances of the Work Processes a node or a publication carries.
/// </summary>
/// <param name="WorkProcesses">Each Work Process's, by node and Work
/// Process.</param>
public sealed record WorkProcessInstanceList(IReadOnlyList<WorkProcessInstances> WorkProcesses)
{
    /// <summary>No Work Process.</summary>
    public static WorkProcessInstanceList Empty { get; } = new([]);

    /// <summary>Every Instance listed, Work Process by Work Process.</summary>
    public IEnumerable<WorkProcessInstanceRecord> Instances =>
        WorkProcesses.SelectMany(process => process.Instances);

    /// <summary>The list section 29 wrote.</summary>
    public static WorkProcessInstanceList Parse(ReadOnlyMemory<byte> json)
    {
        using JsonDocument answer = JsonDocument.Parse(json);

        return new WorkProcessInstanceList(
        [
            .. answer.RootElement.GetProperty("work_process_instances")
                .EnumerateArray()
                .Select(Process),
        ]);
    }

    private static WorkProcessInstances Process(JsonElement process)
    {
        string node = Text(process, "node");
        string name = Text(process, "work_process");

        return new WorkProcessInstances(
            node,
            name,
            process.GetProperty("failed_count").GetUInt64(),
            [
                .. process.GetProperty("failed").EnumerateArray().Select(failed =>
                    new FailedMessageRecord(
                        node,
                        string.Empty,
                        Text(failed, "journey"),
                        failed.GetProperty("sequence").GetUInt64(),
                        Text(failed, "reason"))),
            ],
            [.. process.GetProperty("instances").EnumerateArray().Select(one => Instance(
                node, name, one))]);
    }

    private static WorkProcessInstanceRecord Instance(string node, string name, JsonElement one)
    {
        JsonElement until = one.GetProperty("until_unix_nanos");

        return new WorkProcessInstanceRecord(
            node,
            name,
            Text(one, "instance"),
            Text(one, "state"),
            Text(one, "waits_for"),
            until.ValueKind == JsonValueKind.Number ? Operator.FromNanos(until.GetInt64()) : null,
            Pairs(one.GetProperty("correlations")),
            Text(one, "code"),
            Text(one, "reason"),
            Text(one, "failed_by"),
            Pairs(one.GetProperty("values")),
            [
                .. one.GetProperty("history").EnumerateArray().Select(entry =>
                    new InstanceHandledRecord(
                        Operator.FromNanos(entry.GetProperty("at_unix_nanos").GetInt64()),
                        Text(entry, "what"),
                        Text(entry, "message"),
                        Text(entry, "journey"))),
            ],
            one.GetProperty("earlier").GetUInt64(),
            one.GetProperty("turns").GetUInt64(),
            Operator.FromNanos(one.GetProperty("updated_unix_nanos").GetInt64()));
    }

    private static string Text(JsonElement element, string name)
    {
        return element.GetProperty(name).GetString() ?? string.Empty;
    }

    private static KeyValuePair<string, string>[] Pairs(JsonElement pairs)
    {
        return DeadMessageList.Pairs(pairs);
    }
}
