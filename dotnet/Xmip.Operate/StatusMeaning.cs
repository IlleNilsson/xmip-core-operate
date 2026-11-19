namespace Xmip.Operate;

/// <summary>
/// What a status code means: its name in the header and one line of
/// English, whether trying again could succeed and whether the instance is
/// finished — abi's <c>status_meaning</c>, asked through
/// <c>xmip_status_meaning_v1</c> (<see cref="RuntimeWords.Meaning"/>).
/// <c>xmip status</c> renders the same answer and <c>ConvertFrom-XmipStatus</c>
/// emits it. Until 2026-11-17 the .NET binding wrote its own English.
/// </summary>
/// <param name="Code">The code asked about.</param>
/// <param name="Known">Whether the header defines it.</param>
/// <param name="Name">Its name in the header, or <see cref="Unknown"/>.</param>
/// <param name="Meaning">One line an operator can read.</param>
/// <param name="Retryable">Whether trying again could plausibly succeed.</param>
/// <param name="Terminal">Whether the module instance is finished.</param>
public sealed record StatusMeaning(
    int Code, bool Known, string Name, string Meaning, bool Retryable, bool Terminal)
{
    /// <summary>The name of a code the header does not define.</summary>
    public const string Unknown = "unknown";
}
