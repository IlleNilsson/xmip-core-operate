namespace Xmip.Surface.Test;

/// <summary>
/// A folder of a test's own under the system's temporary directory: the one
/// way a .NET test makes one (ADR-0021, amendment 2026-11-20), as
/// <c>file::temporary::Folder</c> is a Rust test's. One file, compiled into
/// every .NET test project that makes a folder (each links it), so there is
/// one rule.
/// </summary>
/// <remarks>
/// <see cref="Directory.CreateTempSubdirectory(string)"/> makes it,
/// <c>xmip-&lt;label&gt;-&lt;random&gt;</c>, the random part the operating
/// system's, so no other account knows the name before it exists; it is made
/// only where nothing is (Windows' <c>CreateDirectory</c> refuses a folder
/// already there and another name is drawn; Unix's <c>mkdtemp</c> refuses it
/// too, and gives only its owner entry). Disposing it removes it with
/// everything in it, also when the test fails: a <c>using</c> disposes it as
/// the exception leaves, and xUnit disposes a test class whatever its test
/// did. A test makes its folder first, so that it is disposed last, after
/// every file, host or watcher the test opened in it: Windows removes no file
/// that is still open. Where something must not exist yet, the test names a
/// path inside its folder.
/// </remarks>
/// <param name="label">What the folder is for, such as <c>gui-secrets</c>:
/// it is made as <c>xmip-&lt;label&gt;-&lt;random&gt;</c> under the system's
/// temporary directory.</param>
internal sealed class TestFolder(string label) : IDisposable
{
    /// <summary>Where it is.</summary>
    public string Path { get; } = Directory.CreateTempSubdirectory($"xmip-{label}-").FullName;

    /// <summary>The path called <paramref name="name"/> inside it, made by
    /// nothing.</summary>
    public string Inside(string name)
    {
        return System.IO.Path.Join(Path, name);
    }

    /// <inheritdoc />
    public override string ToString()
    {
        return Path;
    }

    /// <summary>Removes it with everything in it. Never throws: a throw while
    /// a failing test's exception leaves would replace that exception. A
    /// folder that cannot be removed is named on standard error.</summary>
    public void Dispose()
    {
        try
        {
            Directory.Delete(Path, recursive: true);
        }
        catch (DirectoryNotFoundException)
        {
            // The test removed it itself: nothing is left.
        }
        catch (Exception error) when (error is IOException or UnauthorizedAccessException)
        {
            Console.Error.WriteLine($"a test's folder {Path} is left: {error.Message}");
        }
    }
}
