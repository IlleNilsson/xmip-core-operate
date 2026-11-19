namespace Xmip.Surface;

/// <summary>
/// What a scope argument selects: the one scope an operator spelled, or the
/// several a wildcard names. ADR-0059 clause 7 is an estate rule, not a
/// PowerShell one — an argument that <em>selects among scopes that already
/// exist</em> takes a wildcard — and the match is <see cref="ScopePattern"/>
/// (ADR-0052, amendment 2026-09-19). One selection for every surface that
/// takes a scope from an operator: <c>xmip</c>'s health, measure, list,
/// show, pause and resume, and <c>Get-XmipHealth</c>, <c>Suspend-XmipScope</c>
/// and <c>Resume-XmipScope</c>. Until 2026-09-24 it lived in the executable
/// and the cmdlets took a scope literally.
/// </summary>
/// <param name="Argument">What the operator typed.</param>
/// <param name="Patterned">Whether it carried a wildcard. An argument without
/// one is the scope itself, answered exactly as it always was.</param>
/// <param name="Scopes">The scopes to answer for, topmost first and never one
/// beneath another: every command reads a scope <em>and everything beneath
/// it</em>, so a child of a match would be said twice.</param>
public sealed record ScopeSelection(
    string Argument, bool Patterned, IReadOnlyList<string> Scopes)
{
    /// <summary>The one scope an argument names, taken literally.</summary>
    public static ScopeSelection Exactly(string scope)
    {
        return new ScopeSelection(scope, false, [scope]);
    }

    /// <summary>
    /// What an argument selects against a surface, or null with the refusal in
    /// <paramref name="refusal"/> when a pattern names nothing at all. A
    /// pattern that matches nothing is never success and never silence:
    /// silence reads as <em>all clear</em>, and this is the opposite of that.
    /// An empty argument is where the publisher publishes, the cluster
    /// (<see cref="IOperatorSurface.Root"/>).
    /// </summary>
    /// <remarks>Whether it is a pattern, the topmost of what it matches and
    /// the refusal are <c>operate::surface::ScopeSelection::over</c>'s, asked
    /// of the runtime over the surface's tree
    /// (<c>xmip_scope_index_select_v1</c>).</remarks>
    public static ScopeSelection? Of(
        IOperatorSurface surface, string argument, out string refusal)
    {
        ArgumentNullException.ThrowIfNull(surface);

        string pattern = string.IsNullOrEmpty(argument) ? surface.Root() : argument;

        return surface.Index().Select(pattern, surface.Source, out refusal);
    }
}
