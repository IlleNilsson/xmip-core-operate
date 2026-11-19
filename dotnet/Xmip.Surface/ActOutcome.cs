using Xmip.Abi.Module;

namespace Xmip.Surface;

/// <summary>
/// What came of an operator's act at a node's door (ADR-0014, amendment
/// 2026-11-13): taken by the node it is asked of; asked — written to Xmip
/// Storage and not taken yet within the door's bound, its node taking it as
/// soon as it runs; or refused, by role, by the door or by the node. Every
/// act's record carries it, so the command line, the PowerShell module and
/// the GUIs say the same word. Its values are the header's
/// <c>XMIP_ACTED_*</c>, and what a status comes to, its word and its
/// sentence are the runtime's (<c>operate::surface::acted</c>, section 28.3).
/// </summary>
public enum ActOutcome
{
    /// <summary>Taken by the node it is asked of.</summary>
    Taken,

    /// <summary>Asked of its node and not taken yet; never lost.</summary>
    Asked,

    /// <summary>Not taken: refused, invalid, failed or unanswered.</summary>
    Refused,
}

/// <summary>The outcome a status of <c>xmip_remote_act_v1</c> says, and its
/// word.</summary>
public static class ActOutcomes
{
    /// <summary>
    /// <see cref="ActOutcome.Taken"/> for <c>XMIP_OK</c>,
    /// <see cref="ActOutcome.Asked"/> for <c>XMIP_E_AGAIN</c> — asked, not a
    /// failure — and <see cref="ActOutcome.Refused"/> for any other.
    /// </summary>
    public static ActOutcome Of(XmipStatus status)
    {
        return (ActOutcome)RuntimeLibrary.Rules.Sentences.ActOutcome((int)status).Outcome;
    }

    /// <summary>The word a surface says it by: <c>taken</c>, <c>asked</c>,
    /// <c>refused</c> — <c>ActOutcome::word</c>, the runtime's.</summary>
    public static string Word(this ActOutcome outcome)
    {
        return RuntimeLibrary.Rules.Sentences.OutcomeWord((int)outcome);
    }

    /// <summary>
    /// What a command says of an act that came to <paramref name="outcome"/>,
    /// saying <paramref name="result"/>, its outcome first in words:
    /// <c>OK.</c>, <c>ASKED.</c> or <c>REFUSED:</c> — <c>ActOutcome::said</c>,
    /// the runtime's.
    /// </summary>
    public static string Said(ActOutcome outcome, string result)
    {
        return RuntimeLibrary.Rules.Sentences.ActSaid((int)outcome, result);
    }
}
