using System.Text.Json;
using Xmip.Abi.Operate;
using Xmip.Operate;

namespace Xmip.Surface;

/// <summary>
/// What a surface asks of the Work Processes' Instances it lists
/// (runtime-model.md section 22; ADR-0066, amendment 2026-11-25), in the words
/// every surface uses: the Instances view's address and
/// <c>Get-XmipInstance</c>'s parameters are these names, as <c>xmip
/// instances</c>'s are <c>--location</c>, its argument and
/// <c>--instance</c>. Where the drill stands — a cluster or a node, and every
/// Work Process run at or beneath it — the Work Process and one Instance:
/// written once, in the runtime (<c>operate::surface::instance_query</c>),
/// which <c>xmip instances</c> asks and every .NET surface asks here.
/// </summary>
public sealed record WorkProcessInstanceQuery
{
    /// <summary>Where the drill stands: a cluster or a node, and every Work
    /// Process run at or beneath it.</summary>
    public string? Location { get; init; }

    /// <summary>One Work Process, by its configured name.</summary>
    public string? WorkProcess { get; init; }

    /// <summary>One Instance, by its identifier.</summary>
    public string? Instance { get; init; }

    /// <summary>
    /// The Work Processes of <paramref name="listed"/> this query selects, in
    /// the order listed, each with only the Instance it names where it names
    /// one; a Work Process left with none of it is left out. The runtime's
    /// choice (<c>XMIP_LIST_INSTANCES</c>).
    /// </summary>
    public IReadOnlyList<WorkProcessInstances> Apply(WorkProcessInstanceList listed)
    {
        ArgumentNullException.ThrowIfNull(listed);

        IReadOnlyList<WorkProcessInstances> processes = listed.WorkProcesses;
        (int Place, int[] Instances)[] chosen = ListQuery.Answer<
            WorkProcessInstances, (int, int[])[]>(
            OperateAbi.List.Instances,
            (processes, [.. processes]),
            ("work_process_instances", Write),
            writer =>
            {
                ListQuery.Text(writer, "location", Location);
                ListQuery.Text(writer, "work_process", WorkProcess);
                ListQuery.Text(writer, "instance", Instance);
            },
            Chosen);

        return [.. chosen.Select(one => Narrowed(processes[one.Place], one.Instances))];
    }

    /// <summary>The one Instance this query names among
    /// <paramref name="listed"/>, with its Work Process; null where it names
    /// none or none is listed.</summary>
    public (WorkProcessInstances Process, WorkProcessInstanceRecord Instance)? One(
        WorkProcessInstanceList listed)
    {
        return string.IsNullOrWhiteSpace(Instance)
            ? null
            : Apply(listed)
                .Select(process => (process, process.Instances[0]))
                .Cast<(WorkProcessInstances, WorkProcessInstanceRecord)?>()
                .FirstOrDefault();
    }

    // A Work Process with the Instances chosen of it, itself where every one
    // was: the runtime keeps their order.
    private static WorkProcessInstances Narrowed(WorkProcessInstances process, int[] places)
    {
        return places.Length == process.Instances.Count
            ? process
            : process with { Instances = [.. places.Select(at => process.Instances[at])] };
    }

    // The answer: each chosen Work Process's place, and the places of its
    // chosen Instances, side by side.
    private static (int, int[])[] Chosen(JsonElement answer)
    {
        int[][] instances =
        [
            .. answer.GetProperty("instances").EnumerateArray()
                .Select(places => places.EnumerateArray().Select(at => at.GetInt32()).ToArray()),
        ];

        return
        [
            .. ListQuery.Places(answer, "places").Select((place, at) => (place, instances[at])),
        ];
    }

    // One Work Process in section 29's JSON: the members the query reads.
    private static void Write(Utf8JsonWriter writer, WorkProcessInstances process)
    {
        writer.WriteString("node", process.Node);
        writer.WriteString("work_process", process.WorkProcess);
        writer.WriteStartArray("instances");
        foreach (WorkProcessInstanceRecord one in process.Instances)
        {
            writer.WriteStartObject();
            writer.WriteString("instance", one.Instance);
            writer.WriteEndObject();
        }

        writer.WriteEndArray();
    }
}
