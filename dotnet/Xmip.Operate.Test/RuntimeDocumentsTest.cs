using System.Text.Json;
using System.Text.Json.Nodes;

namespace Xmip.Operate.Test;

/// <summary>
/// <see cref="RuntimeDocuments"/> crosses section 27 of
/// <c>xmip_operate.h</c> to the runtime this estate built. Reading, writing
/// and setting are tested in Rust, where they are written
/// (<c>xmip-core-configure</c>); these prove the crossing: each answer whole
/// and in the header's shape, and a refusal carried as the reader's
/// sentence.
/// </summary>
public sealed class RuntimeDocumentsTest
{
    private const string Document = "# kept\n[Xmip]\nSurface = \"local\" # trails\n";

    [Fact]
    public void ADocumentComesBackAsTheHeadersJson()
    {
        Assert.True(RuntimeRulesTest.Rules.Documents.TryRead(Document, out string answer), answer);

        using JsonDocument read = JsonDocument.Parse(answer);

        Assert.Equal(
            "local", read.RootElement.GetProperty("Xmip").GetProperty("Surface").GetString());
    }

    [Fact]
    public void JsonIsWrittenAsTomlThatReadsBack()
    {
        const string json = """{"top":{"word":"said"},"outer":{"inner":{"flag":true}}}""";

        Assert.True(RuntimeRulesTest.Rules.Documents.TryWrite(json, out string text), text);
        Assert.Contains("[outer.inner]", text, StringComparison.Ordinal);
        Assert.True(RuntimeRulesTest.Rules.Documents.TryRead(text, out string back), back);
        Assert.True(JsonNode.DeepEquals(JsonNode.Parse(json), JsonNode.Parse(back)), back);
    }

    [Fact]
    public void OneKeyIsSetAndTheRestStaysAsWritten()
    {
        Assert.True(
            RuntimeRulesTest.Rules.Documents.TrySet(
                Document, ["Xmip"], "Surface", JsonValue.Create("remote"), out string set),
            set);

        Assert.Equal(Document.Replace("local", "remote", StringComparison.Ordinal), set);
    }

    [Fact]
    public void TextThatIsNotTomlIsRefusedInTheReadersWords()
    {
        Assert.False(RuntimeRulesTest.Rules.Documents.TryRead("[Xmip", out string refusal));
        Assert.StartsWith("configuration parse failed", refusal, StringComparison.Ordinal);
    }
}
