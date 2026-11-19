using System.Text.Json;
using System.Text.Json.Nodes;
using Xmip.Abi.Operate;
using Xmip.Operate;

namespace Xmip.Surface;

/// <summary>
/// What an Operator does to a Message that failed (runtime-model.md section
/// 13; ADR-0013): <c>observe::Act</c> as <c>observe::Noun::Message</c> takes
/// it, whose word — the name, lower case — is what crosses to the runtime.
/// </summary>
public enum MessageAct
{
    /// <summary>Write it Active, its tries begun anew, and send it again from
    /// the end of its Send Port's queue — or from its place, where it blocks
    /// its sequence.</summary>
    Retry,

    /// <summary>Write it Dismissed — terminal, its history, Message and
    /// Stream kept — and take it out of its Send Port's queue.</summary>
    Dismiss,
}

/// <summary>
/// What came of a <see cref="MessageAct"/> on the Message whose Journey is
/// <paramref name="Journey"/> sent by the node at <paramref name="Node"/>:
/// whether it was applied, and what was said. The executable and the
/// PowerShell module emit this one shape, as they emit a
/// <see cref="DeadMessageOperation"/>.
/// </summary>
public sealed record MessageOperation(
    string Node, string Journey, MessageAct Act, bool Applied, string Result)
{
    /// <summary>What came of it: taken where it was applied, refused where
    /// it was not, unless the door said asked (<see cref="ActOutcome"/>).</summary>
    public ActOutcome Outcome { get; init; } = Applied ? ActOutcome.Taken : ActOutcome.Refused;

    /// <summary>The noun an order on a failed Message is left under:
    /// <c>observe::Noun::Message</c>'s word.</summary>
    public const string Noun = "message";

    /// <summary>The word the runtime names <paramref name="act"/> by.</summary>
    public static string Word(MessageAct act)
    {
        return RuntimeLibrary.Rules.Sentences.ActWord(act switch
        {
            MessageAct.Retry => OperateAbi.Act.Retry,
            MessageAct.Dismiss => OperateAbi.Act.Dismiss,
            _ => throw new ArgumentOutOfRangeException(nameof(act), act, null),
        });
    }

    /// <summary>
    /// The node and the Send Port a scope names: the node it is on
    /// (<see cref="ScopeTree.NodeScope"/>) and the segment after its stage,
    /// where it is that deep — empty for each it does not name. The
    /// runtime's rule (section 28.2), the one a surface lists by.
    /// </summary>
    public static (string Node, string Port) PortAt(string scope)
    {
        ArgumentNullException.ThrowIfNull(scope);

        using JsonDocument at = JsonDocument.Parse(RuntimeLibrary.Rules.Surfaces.PortAt(scope));

        return (
            at.RootElement.GetProperty("node").GetString() ?? string.Empty,
            at.RootElement.GetProperty("port").GetString() ?? string.Empty);
    }

    /// <summary>
    /// The Messages that failed at <paramref name="scope"/>, as
    /// <paramref name="read"/> lists them — or, where Xmip Storage did not
    /// answer it, <see cref="FailedMessageList.Unanswered"/> in the runtime's
    /// words: said, not thrown, so a screen or a command shows it and goes on.
    /// </summary>
    public static FailedMessageList Read(string scope, Func<FailedMessageList> read)
    {
        ArgumentNullException.ThrowIfNull(read);

        try
        {
            return read();
        }
        catch (InvalidOperationException unanswered)
        {
            return FailedMessageList.Unanswered(
                $"the Messages that failed at {scope} could not be read: {unanswered.Message}");
        }
    }

    /// <summary>
    /// What a list of the Messages that failed at <paramref name="scope"/>
    /// says where it is no answer, read from <paramref name="source"/>: what
    /// the surface was told where it asked (<c>FAILED:</c>), or that it
    /// cannot list them, so nothing is known of the queue (<c>NOT
    /// LISTED:</c>). Empty where the list is an answer. The one sentence
    /// every surface says it with, the runtime's (section 28.2).
    /// </summary>
    public static string Unlisted(FailedMessageList failed, string scope, string source)
    {
        ArgumentNullException.ThrowIfNull(failed);

        return RuntimeLibrary.Rules.Surfaces.Unlisted(new JsonObject
        {
            ["listed"] = failed.Listed,
            ["failure"] = failed.Failure,
            ["scope"] = scope,
            ["source"] = source,
        }.ToJsonString());
    }

    /// <summary>The act not taken, and why.</summary>
    public static MessageOperation Declined(
        string node, string journey, MessageAct act, string why)
    {
        return new MessageOperation(node, journey, act, false, why);
    }

    /// <summary>
    /// The last Message that failed at a Send Port, read from the evidence
    /// its node publishes at the Port's scope — the clause
    /// <c>; the Message of the Journey &lt;id&gt; failed: &lt;why&gt;</c> the runtime's
    /// <c>send_step::PortFigures::evidence</c> writes — or null where the
    /// evidence names none. The one reader of that clause; every surface
    /// that offers an act on the Message asks it.
    /// </summary>
    public static string? FailedIn(string? evidence)
    {
        const string Opening = "; the Message of the Journey ";
        const string Closing = " failed: ";

        int start = evidence?.IndexOf(Opening, StringComparison.Ordinal) ?? -1;

        if (evidence is null || start < 0)
        {
            return null;
        }

        start += Opening.Length;
        int end = evidence.IndexOf(Closing, start, StringComparison.Ordinal);

        return end > start && !evidence.AsSpan(start, end - start).ContainsAny(" ;")
            ? evidence[start..end]
            : null;
    }

    /// <summary>
    /// How many Messages that failed wait in a Send Port's queue now, read
    /// from the evidence its node publishes at the Port's scope — the figure
    /// <c>failed in its queue &lt;n&gt;</c> the runtime's
    /// <c>send_step::PortFigures::evidence</c> writes — or null where the
    /// evidence says none. Unlike <see cref="FailedIn"/>, which names the
    /// last Message that ever failed there, this is the queue as it stands:
    /// zero once every failed Message was retried or dismissed.
    /// </summary>
    public static ulong? FailingIn(string? evidence)
    {
        const string Opening = "failed in its queue ";

        int start = evidence?.IndexOf(Opening, StringComparison.Ordinal) ?? -1;

        if (evidence is null || start < 0)
        {
            return null;
        }

        start += Opening.Length;
        int end = start;

        while (end < evidence.Length && char.IsAsciiDigit(evidence[end]))
        {
            end++;
        }

        return ulong.TryParse(
            evidence.AsSpan(start, end - start),
            System.Globalization.NumberStyles.None,
            System.Globalization.CultureInfo.InvariantCulture,
            out ulong failing)
            ? failing
            : null;
    }
}
