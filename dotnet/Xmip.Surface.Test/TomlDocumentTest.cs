using Microsoft.Extensions.Configuration;

namespace Xmip.Surface.Test;

/// <summary>
/// A document read and one key of it set by the runtime's one reader,
/// <c>xmip-core-configure</c>, through <c>xmip_operate.h</c> section 27: a
/// value changes and nothing else does (open problem 25, rows b and u).
/// </summary>
public sealed class TomlDocumentTest
{
    private const string Document = """
        # heading
        [service]
        name = "a" # trailing

        # before the rows
        [[row]]
        key = "one"

        [other]
        "dotted.key" = true
        whole = 7

        """;

    [Fact]
    public void ADocumentReadsAsTheConfigurationsKeysArraysByPosition()
    {
        using Papers papers = new(Document);

        IConfigurationRoot read = TomlDocument.Read(papers.Path);

        Assert.Equal("a", read["service:name"]);
        Assert.Equal("one", read["row:0:key"]);
        Assert.Equal("true", read["other:dotted.key"]);
        Assert.Equal("7", read["other:whole"]);
    }

    [Fact]
    public void AFileThatIsNotThereReadsAsAnEmptyDocument()
    {
        using TestFolder folder = new("absent");

        Assert.Empty(TomlDocument.Read(folder.Inside("absent.toml")).AsEnumerable());
    }

    [Fact]
    public void AFileThatIsNotTomlIsRefusedInTheReadersWords()
    {
        using Papers papers = new("[service\nname =");

        InvalidDataException refused =
            Assert.Throws<InvalidDataException>(() => TomlDocument.Read(papers.Path));

        Assert.IsType<FormatException>(refused.InnerException);
        Assert.StartsWith(
            "configuration parse failed", refused.InnerException.Message, StringComparison.Ordinal);
    }

    [Fact]
    public void ASetReplacesTheValueAndKeepsItsComment()
    {
        string written = TomlDocument.Set(Document, ["service"], "name", "b");

        Assert.Equal(
            Document.Replace("name = \"a\"", "name = \"b\"", StringComparison.Ordinal), written);
    }

    [Fact]
    public void ASetOfAMissingTableAddsItAndARemovalLeavesTheRest()
    {
        string added = TomlDocument.Set(Document, ["bar"], "order", "audit,board");

        Assert.StartsWith(Document, added, StringComparison.Ordinal);
        Assert.EndsWith("[bar]\norder = \"audit,board\"\n", added, StringComparison.Ordinal);
        Assert.Equal(Document, TomlDocument.Set(Document, ["absent"], "order", null));
    }

    [Fact]
    public void TextThatIsNotTomlIsAFormatException()
    {
        Assert.Throws<FormatException>(() => TomlDocument.Set("[service\nname =", [], "a", "b"));
    }

    private sealed class Papers : IDisposable
    {
        private readonly TestFolder folder = new("document");

        public Papers(string text)
        {
            Path = folder.Inside("document.toml");
            File.WriteAllText(Path, text);
        }

        public string Path { get; }

        public void Dispose()
        {
            folder.Dispose();
        }
    }
}
