using System.Text.Json.Nodes;
using Microsoft.Extensions.Configuration;

namespace Xmip.Surface;

/// <summary>
/// The one way a .NET surface reads and edits a TOML document (ADR-0052
/// clause 1). On disk the estate is TOML — a host's <c>gui.toml</c>, a node
/// configuration, a published snapshot — and JSON is reserved for memory and
/// the wire; so there is one reader, and no host adds a JSON source. That
/// reader is the runtime's, <c>xmip-core-configure</c>, reached through
/// <c>xmip_operate.h</c> section 27 (<see cref="TomlConfigurationProvider"/>;
/// ADR-0031, amendment 2026-11-17): until that day .NET read TOML with a
/// package of its own.
/// </summary>
/// <remarks>
/// A surface that edits a document — the Desktop's preferences — sets one key
/// of its text here (<see cref="Set"/>): the value is replaced in place and
/// everything else, comments, blank lines and the order of keys and tables
/// included, is written back exactly as it was read, the comment trailing
/// the old value trailing the new one. Keys are matched whole, and strings
/// are quoted by the runtime's writer, never by hand.
/// </remarks>
public static class TomlDocument
{
    /// <summary>Read one document. A file that is not there reads as an empty
    /// document; whoever asked decides whether that is a problem.</summary>
    /// <exception cref="InvalidDataException">The file is not TOML; its inner
    /// exception says where, in the reader's words.</exception>
    public static IConfigurationRoot Read(string path)
    {
        return Add(new ConfigurationBuilder(), path, optional: true, watched: false).Build();
    }

    /// <summary>Add a document to a host's configuration, watched for change.
    /// A relative path resolves the way the host's builder resolves files —
    /// its content root.</summary>
    public static IConfigurationBuilder Add(
        IConfigurationBuilder builder, string path, bool optional)
    {
        return Add(builder, path, optional, watched: true);
    }

    /// <summary>
    /// The document <paramref name="text"/> with the key
    /// <paramref name="key"/> of the table <paramref name="table"/> names set
    /// to the string <paramref name="value"/>, or removed where it is null;
    /// everything else as written. A table missing on the way is added when a
    /// value is set.
    /// </summary>
    /// <exception cref="FormatException">The text is not TOML, or a key on
    /// the way holds a value; the message says why, in the reader's
    /// words.</exception>
    public static string Set(string text, IReadOnlyList<string> table, string key, string? value)
    {
        return RuntimeLibrary.Rules.Documents.TrySet(
            text, table, key, value is null ? null : JsonValue.Create(value), out string answer)
            ? answer
            : throw new FormatException(answer);
    }

    /// <summary>A path a document names: absolute as written, relative joined
    /// to <paramref name="basePath"/>, which is the directory the document's
    /// paths are written from.</summary>
    public static string Resolve(string path, string basePath)
    {
        return Path.IsPathRooted(path)
            ? Path.GetFullPath(path)
            : Path.GetFullPath(Path.Combine(basePath, path));
    }

    /// <summary>
    /// The directory a GUI host's document writes its paths from: the estate
    /// root, the directory holding <c>architecture.toml</c>, when the host
    /// runs from inside the estate, else <paramref name="here"/>, the host's
    /// own directory once published. One rule for the desktop and the web:
    /// until 2026-09-28 the web resolved against its content root, which is
    /// the project under <c>dotnet run</c> and its bin folder when started
    /// built, and the same path found no runtime library.
    /// </summary>
    public static string BasePath(string here)
    {
        for (DirectoryInfo? directory = new(here);
            directory is not null;
            directory = directory.Parent)
        {
            if (File.Exists(Path.Combine(directory.FullName, "architecture.toml")))
            {
                return directory.FullName;
            }
        }

        return here;
    }

    private static IConfigurationBuilder Add(
        IConfigurationBuilder builder, string path, bool optional, bool watched)
    {
        ArgumentNullException.ThrowIfNull(builder);

        return builder.Add<TomlConfigurationSource>(source =>
        {
            source.Path = path;
            source.Optional = optional;
            source.ReloadOnChange = watched;
            source.ResolveFileProvider();
        });
    }
}
