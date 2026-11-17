using System.Globalization;
using System.Text;
using System.Text.Json;
using Microsoft.Extensions.Configuration;

namespace Xmip.Surface;

/// <summary>
/// A TOML document's keys and values as a host's configuration, read by
/// <c>xmip-core-configure</c> through the runtime's library
/// (<c>xmip_operate.h</c> section 27; ADR-0031, amendment 2026-11-17): what
/// the runtime reads is what a surface reads, by one parser. A table's keys
/// are joined by the configuration's separator, an array's entries keyed by
/// their position, and a value is its text.
/// </summary>
/// <param name="source">Where the document is and whether it is
/// watched.</param>
public sealed class TomlConfigurationProvider(TomlConfigurationSource source)
    : FileConfigurationProvider(source)
{
    /// <inheritdoc />
    /// <exception cref="FormatException">The text is not TOML, in the
    /// reader's words; the file provider says which file.</exception>
    public override void Load(Stream stream)
    {
        using StreamReader reader = new(stream, Encoding.UTF8);

        if (!RuntimeLibrary.Rules.Documents.TryRead(reader.ReadToEnd(), out string answer))
        {
            throw new FormatException(answer);
        }

        Dictionary<string, string?> data = [with(StringComparer.OrdinalIgnoreCase)];

        using (JsonDocument document = JsonDocument.Parse(answer))
        {
            Flatten(document.RootElement, null, data);
        }

        Data = data;
    }

    private static void Flatten(JsonElement element, string? path, Dictionary<string, string?> data)
    {
        switch (element.ValueKind)
        {
            case JsonValueKind.Object:
                foreach (JsonProperty property in element.EnumerateObject())
                {
                    Flatten(property.Value, Joined(path, property.Name), data);
                }

                break;
            case JsonValueKind.Array:
                int position = 0;
                foreach (JsonElement entry in element.EnumerateArray())
                {
                    string key = position++.ToString(CultureInfo.InvariantCulture);
                    Flatten(entry, Joined(path, key), data);
                }

                break;
            case JsonValueKind.String:
                data[path ?? string.Empty] = element.GetString();
                break;
            case JsonValueKind.Number:
            case JsonValueKind.True:
            case JsonValueKind.False:
                data[path ?? string.Empty] = element.GetRawText();
                break;
            case JsonValueKind.Undefined:
            case JsonValueKind.Null:
            default:
                // The reader writes neither: TOML has no null.
                break;
        }
    }

    private static string Joined(string? path, string key)
    {
        return path is null ? key : ConfigurationPath.Combine(path, key);
    }
}
