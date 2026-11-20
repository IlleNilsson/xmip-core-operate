using Xmip.Operate;

namespace Xmip.Surface.Test;

/// <summary>
/// One rule for finding the runtime, in order: a library stated for the
/// invocation, the environment, beside the executable (ADR-0052 clause 1;
/// amendment 2026-11-17: a surface's document no longer names it, since the
/// runtime reads the document).
/// </summary>
public sealed class RuntimeLibraryTest : IDisposable
{
    // Names alone: nothing is made at either.
    private readonly TestFolder folder = new("surface-test");

    private string Base => folder.Inside("base");

    private string Beside => folder.Inside("exe");

    public void Dispose()
    {
        folder.Dispose();
    }

    [Fact]
    public void TheEnvironmentIsFirst()
    {
        string fromEnvironment = Path.Combine(Base, "env-runtime.dll");

        Assert.Equal(fromEnvironment, RuntimeLibrary.Choose(fromEnvironment, Beside));
    }

    [Fact]
    public void BesideTheExecutableIsLast()
    {
        string beside = Path.Combine(Beside, NativeRuntime.FileName);

        Assert.Equal(beside, RuntimeLibrary.Choose("", Beside));
        Assert.Equal(beside, RuntimeLibrary.Choose(null, Beside));
    }

    [Fact]
    public void ALibraryStatedForTheInvocationWinsOverEverything()
    {
        // A cmdlet's -Library: the one precedence, taken from the current
        // directory.
        Assert.Equal(
            Path.GetFullPath("override.dll"), RuntimeLibrary.Stated("override.dll", Beside));
    }

    [Fact]
    public void ABlankStatementIsNoStatementAndTheRuleDecides()
    {
        string ruled = RuntimeLibrary.Choose(
            Environment.GetEnvironmentVariable(RuntimeLibrary.EnvironmentVariable), Beside);

        Assert.Equal(ruled, RuntimeLibrary.Stated(" ", Beside));
        Assert.Equal(ruled, RuntimeLibrary.Stated(null, Beside));
    }
}
