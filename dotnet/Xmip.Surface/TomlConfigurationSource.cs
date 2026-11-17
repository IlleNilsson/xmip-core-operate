using Microsoft.Extensions.Configuration;

namespace Xmip.Surface;

/// <summary>
/// A TOML document as a host's configuration source: the file found and
/// watched by the platform's file provider, its text read by the runtime's
/// one reader (<see cref="TomlConfigurationProvider"/>).
/// </summary>
public sealed class TomlConfigurationSource : FileConfigurationSource
{
    /// <inheritdoc />
    public override IConfigurationProvider Build(IConfigurationBuilder builder)
    {
        EnsureDefaults(builder);
        return new TomlConfigurationProvider(this);
    }
}
