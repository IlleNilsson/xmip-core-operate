namespace Xmip.Surface.Test;

/// <summary>
/// <see cref="TestFolder"/>, the one way a .NET test makes a folder
/// (ADR-0021, amendment 2026-11-20): made under the system's temporary
/// directory, two of one label are two, and disposing it removes it with
/// everything in it, also when the test body throws. On 2026-11-20 about
/// 1,267 folders per label of the GUI's tests were found under this
/// directory, each made and never removed.
/// </summary>
public sealed class TestFolderTest
{
    private const string Label = "temporary-test";

    [Fact]
    public void AFolderIsMadeUnderTheTemporaryDirectoryAndRemovedWithItsContent()
    {
        TestFolder folder = new(Label);
        string made = folder.Path;

        Assert.True(Directory.Exists(made), made);
        Assert.Equal(
            Path.TrimEndingDirectorySeparator(Path.GetTempPath()),
            Path.GetDirectoryName(made));
        Assert.StartsWith($"xmip-{Label}-", Path.GetFileName(made), StringComparison.Ordinal);

        Directory.CreateDirectory(folder.Inside("inner"));
        File.WriteAllText(Path.Join(folder.Inside("inner"), "order.edi"), "UNA");
        folder.Dispose();

        Assert.False(Directory.Exists(made), $"{made} is left");
    }

    [Fact]
    public void TwoFoldersOfOneLabelAreTwo()
    {
        using TestFolder first = new(Label);
        using TestFolder second = new(Label);

        Assert.NotEqual(first.Path, second.Path);
    }

    [Fact]
    public void AFolderIsRemovedAlsoWhenTheTestBodyThrows()
    {
        string? made = null;

        Assert.Throws<InvalidOperationException>((Action)(() =>
        {
            using TestFolder folder = new(Label);
            made = folder.Path;
            File.WriteAllText(folder.Inside("order.edi"), "UNA");

            throw new InvalidOperationException("the test fails");
        }));

        Assert.NotNull(made);
        Assert.False(Directory.Exists(made), $"{made} is left");
    }

    [Fact]
    public void AFolderTheTestRemovedItselfIsDisposedWithoutComplaint()
    {
        TestFolder folder = new(Label);
        Directory.Delete(folder.Path);

        folder.Dispose();

        Assert.False(Directory.Exists(folder.Path));
    }

    [Fact]
    public void APathInsideIsNamedAndNotMade()
    {
        using TestFolder folder = new(Label);
        string inside = folder.Inside("no-such-runtime.dll");

        Assert.Equal(folder.Path, Path.GetDirectoryName(inside));
        Assert.False(File.Exists(inside));
    }
}
