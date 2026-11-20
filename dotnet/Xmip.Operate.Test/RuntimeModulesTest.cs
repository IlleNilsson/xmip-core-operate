using Xmip.Abi.Module;

namespace Xmip.Operate.Test;

/// <summary>
/// <see cref="RuntimeModules"/> crosses section 26 of <c>xmip_operate.h</c>
/// to the runtime this estate built. The opening and the judgment are tested
/// in Rust, in the runtime's loader, where they are written; these prove the
/// crossing: a library that cannot be asked is an answer, whole, and never an
/// exception.
/// </summary>
public sealed class RuntimeModulesTest
{
    [Fact]
    public void ALibraryThatIsNotThereIsAnAnswerAndNotAnException()
    {
        using TestFolder folder = new("operate-module-missing");
        string missing = folder.Inside("no-such-xmip-module.dll");

        ModuleProbed probed = RuntimeRulesTest.Rules.Modules.Probe(missing);

        Assert.False(probed.Loaded);
        Assert.Contains("would not open", probed.Unloadable, StringComparison.Ordinal);
        Assert.Equal(missing, probed.Library);
        Assert.Equal(XmipStatus.NotFound, probed.Status);
        Assert.False(probed.Conforms);
        Assert.Empty(probed.Log);
    }

    [Fact]
    public void AFileThatIsNoLibraryIsUnloadableRatherThanLoaded()
    {
        string notALibrary = Path.Combine(AppContext.BaseDirectory, "Xmip.Operate.Test.deps.json");

        ModuleProbed probed = RuntimeRulesTest.Rules.Modules.Probe(notALibrary);

        Assert.False(probed.Loaded);
        Assert.Equal(string.Empty, probed.Provider);
    }
}
