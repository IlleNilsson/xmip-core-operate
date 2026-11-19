using System.Diagnostics.CodeAnalysis;
using Xmip.Operate;

namespace Xmip.Surface;

/// <summary>
/// Where the runtime's native library is, by one rule for every surface
/// (ADR-0052 clause 1): a library stated for one invocation, else the
/// <c>XMIP_RUNTIME_LIBRARY</c> environment variable, else the library
/// beside the executable — and the one library this process calls the
/// runtime's rules in (<see cref="Rules"/>).
/// </summary>
/// <remarks>
/// The rule is written here and nowhere else in the estate. A surface has to
/// find the runtime before it can call anything in it, so this is the one
/// rule a .NET surface cannot ask the runtime for; the Rust copy the language
/// server kept went on 2026-09-24, and that server is told its library
/// (ADR-0052, amendment 2026-09-24). A surface's own document cannot name
/// it: the runtime reads that document (<see cref="TomlDocument"/>), so the
/// library is loaded before it is read, and its <c>RuntimeLibrary</c> key
/// went on 2026-11-17 (ADR-0052, amendment of that day). On a phone the path
/// is only shown: the library loaded is the one packaged with the
/// application (<see cref="NativeRuntime"/>; ADR-0014, amendment
/// 2026-10-14).
/// </remarks>
public static class RuntimeLibrary
{
    /// <summary>The environment variable an operator sets once per machine.</summary>
    public const string EnvironmentVariable = "XMIP_RUNTIME_LIBRARY";

    private static readonly Lock Gate = new();

    private static volatile RuntimeRules? loaded;

    private static string? preferred;

    /// <summary>
    /// Where this build of the surface lies — beside the executable for the
    /// command line and the GUI hosts, beside the module for PowerShell, whose
    /// executable is pwsh — and so where the runtime library lies by default.
    /// </summary>
    public static string Beside => AssemblyDirectory() ?? AppContext.BaseDirectory;

    // Xmip.Surface's own file, where it has one. An ahead-of-time or
    // single-file build has none — its Location is empty, which is what the
    // analyzers warn of — and there the executable's directory is where it
    // lies, which is the answer.
    [UnconditionalSuppressMessage(
        "SingleFile",
        "IL3000:Avoid accessing Assembly file path when publishing as a single file",
        Justification = "An empty Location is answered by AppContext.BaseDirectory.")]
    private static string? AssemblyDirectory()
    {
        string? here = Path.GetDirectoryName(typeof(RuntimeLibrary).Assembly.Location);

        return string.IsNullOrEmpty(here) ? null : here;
    }

    /// <summary>
    /// The runtime's rules — scope containment, the stage words and their
    /// parse, a mood's word and color name, the worst-first order — in the
    /// library this process loads, once. Every surface calls these and keeps
    /// no rule of its own (<c>xmip_operate.h</c> section 7). The library is
    /// the one stated for this invocation (<see cref="Stated"/>), else the
    /// rule with nothing stated.
    /// </summary>
    /// <exception cref="InvalidOperationException">No runtime library could be
    /// loaded; the message says where it looked and how to put one there.</exception>
    public static RuntimeRules Rules
    {
        get
        {
            // Asked thousands of times a render; loaded once, then read freely.
            if (loaded is { } ready)
            {
                return ready;
            }

            lock (Gate)
            {
                if (loaded is not null)
                {
                    return loaded;
                }

                string fallback = Choose(
                    Environment.GetEnvironmentVariable(EnvironmentVariable), Beside);
                string reason = string.Empty;

                foreach (string path in new[] { preferred, fallback }.OfType<string>().Distinct())
                {
                    loaded = RuntimeRules.Load(path, out reason);

                    if (loaded is not null)
                    {
                        return loaded;
                    }
                }

                throw new InvalidOperationException(
                    $"The runtime's rules are not loaded: {reason}. Build it (cargo build in " +
                    $"module/core/operation/operate), or set {EnvironmentVariable}.");
            }
        }
    }


    /// <summary>
    /// The library an operator stated, over the rule: a path typed for this
    /// one invocation — <c>-Library</c> on a cmdlet — wins and is taken from
    /// the current directory; a blank one is no statement, and then the
    /// environment and <paramref name="besideExecutable"/> decide as
    /// <see cref="Choose"/> says. Stated before a document is read, since
    /// reading one loads the library. One precedence for every .NET surface
    /// (ADR-0052 clause 1); the command line links the runtime and states
    /// none (ADR-0014, amendment 2026-11-17).
    /// </summary>
    public static string Stated(string? overridden, string besideExecutable)
    {
        return Prefer(string.IsNullOrWhiteSpace(overridden)
            ? Choose(Environment.GetEnvironmentVariable(EnvironmentVariable), besideExecutable)
            : Path.GetFullPath(overridden));
    }

    /// <summary>The rule itself, with every input in hand: the environment
    /// first, then beside the executable.</summary>
    public static string Choose(string? fromEnvironment, string besideExecutable)
    {
        return !string.IsNullOrWhiteSpace(fromEnvironment)
            ? Path.GetFullPath(fromEnvironment)
            : Path.Combine(besideExecutable, NativeRuntime.FileName);
    }

    // The library stated for this invocation is the one its rules are called
    // in, until they are: the first stated wins, and a path that does not
    // load falls back to the rule with nothing stated. A program's audit
    // (ProgramAudit) states its library through here too.
    internal static string Prefer(string path)
    {
        lock (Gate)
        {
            if (loaded is null)
            {
                preferred ??= path;
            }
        }

        return path;
    }
}
