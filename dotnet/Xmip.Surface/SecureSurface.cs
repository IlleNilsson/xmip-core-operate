using System.Net;
using System.Text.Json.Nodes;
using Microsoft.Extensions.Configuration;

namespace Xmip.Surface;

/// <summary>
/// What a surface presents and what it trusts when it crosses a network
/// (ADR-0063 clause 1: internal traffic is mutual TLS, a loopback connection
/// inside one machine the one exception, said where it is made): the files a
/// host's document names, handed to the runtime, whose TLS is Xmip's own
/// (<c>xmip-core-library-tls</c>). A web host serves its pages over HTTPS in
/// it (<c>xmip_operate.h</c> section 21), presenting this certificate and
/// verifying the one a browser presents against these anchors; a
/// <see cref="RemoteOperator"/> hands the runtime the same paths, and the
/// runtime reads them, presents this certificate to a node's operate
/// listener and checks the node's against these anchors, saying what is
/// missing in words. No .NET TLS stack reads these files.
/// </summary>
/// <remarks>
/// The certificate and its trust come the way <c>xmip-core-library-tls</c>
/// takes them (ADR-0033, ADR-0034): a PEM chain and its PEM private key, and
/// the anchors a peer's chain must reach, a PEM file of them — never the
/// operating system's trust store (ADR-0063). Where the files came from —
/// an internal CA, ACME, the System Tests' roll authority — is provisioning's,
/// not this. In the host's <c>[Xmip]</c> table: <c>Certificate</c>,
/// <c>PrivateKey</c> and <c>TrustAnchor</c>, each else its environment
/// variable. Revocation is not checked here yet: the estate's revocation is
/// by held CRLs (ADR-0045), and none is configured for a surface.
/// </remarks>
public sealed partial class SecureSurface
{
    /// <summary>The key naming this end's PEM certificate chain, leaf first.</summary>
    public const string CertificateKey = "Xmip:Certificate";

    /// <summary>The key naming the PEM private key of that certificate.</summary>
    public const string PrivateKeyKey = "Xmip:PrivateKey";

    /// <summary>The key naming the PEM anchors a peer's chain must reach.</summary>
    public const string TrustAnchorKey = "Xmip:TrustAnchor";

    /// <summary>The environment variable when the document names no certificate.</summary>
    public const string CertificateVariable = "XMIP_CERTIFICATE";

    /// <summary>The environment variable when the document names no private key.</summary>
    public const string PrivateKeyVariable = "XMIP_PRIVATE_KEY";

    /// <summary>The environment variable when the document names no anchors.</summary>
    public const string TrustAnchorVariable = "XMIP_TRUST_ANCHOR";

    private SecureSurface((string? Certificate, string? PrivateKey, string? TrustAnchor) paths)
    {
        Paths = paths;
    }

    /// <summary>Nothing presented and nothing trusted.</summary>
    public static SecureSurface None { get; } = new((null, null, null));

    /// <summary>The files this was read from, each null where none was
    /// named: a web host serves no HTTPS without the three, and a remote
    /// surface presents nothing without the first two.</summary>
    public (string? Certificate, string? PrivateKey, string? TrustAnchor) Paths { get; }

    /// <summary>Whether <paramref name="host"/> is this machine: <c>localhost</c>
    /// or a loopback address. <c>0.0.0.0</c>, <c>*</c> and <c>+</c> are every
    /// interface, and not loopback.</summary>
    public static bool IsLoopback(string? host)
    {
        if (string.IsNullOrWhiteSpace(host))
        {
            return false;
        }

        string bare = host.Trim().TrimStart('[').TrimEnd(']');

        return string.Equals(bare, "localhost", StringComparison.OrdinalIgnoreCase)
            || (IPAddress.TryParse(bare, out IPAddress? address) && IPAddress.IsLoopback(address));
    }

    /// <summary>
    /// Whether a connection to <paramref name="address"/> may be made: HTTPS
    /// always, plain HTTP only on loopback (ADR-0063 clause 1). The reason
    /// says why not, in words.
    /// </summary>
    public static bool Permits(Uri address, out string reason)
    {
        ArgumentNullException.ThrowIfNull(address);

        return Permits(address.Scheme, address.Host, out reason);
    }

    /// <summary>
    /// The one rule for connecting and for binding alike: a scheme of
    /// <c>https</c> always, anything else only where <paramref name="host"/>
    /// is this machine (ADR-0063 clause 1).
    /// </summary>
    public static bool Permits(string scheme, string host, out string reason)
    {
        if (string.Equals(scheme, Uri.UriSchemeHttps, StringComparison.OrdinalIgnoreCase)
            || IsLoopback(host))
        {
            reason = string.Empty;
            return true;
        }

        reason = $"REFUSED. {host} is plain {scheme} beyond this machine; Xmip's own traffic "
            + "is TLS, loopback the one exception (ADR-0063 clause 1)";
        return false;
    }

    /// <summary>
    /// The certificate, key and anchors the configuration names, each key
    /// else its environment variable; paths resolve against
    /// <paramref name="basePath"/>. None of them named is <see cref="None"/>.
    /// </summary>
    /// <exception cref="InvalidOperationException">A certificate without its
    /// key or the reverse, or a file that is not there; the message names
    /// it.</exception>
    public static SecureSurface From(IConfiguration configuration, string basePath)
    {
        return From(configuration, basePath, null);
    }

    /// <summary>
    /// As <see cref="From(IConfiguration, string)"/>, where this end's key
    /// may be kept by its platform's <paramref name="keeper"/> instead of a
    /// file: a configuration that names a certificate and no private key
    /// presents that chain with the key the keeper holds
    /// (<see cref="Kept"/>). A configuration naming a private key reads it,
    /// keeper or not.
    /// </summary>
    /// <exception cref="InvalidOperationException">As
    /// <see cref="From(IConfiguration, string)"/>.</exception>
    public static SecureSurface From(
        IConfiguration configuration, string basePath, IOperatorKeeper? keeper)
    {
        ArgumentNullException.ThrowIfNull(configuration);

        // Which files the document names, else the environment, and whether
        // they stand together: the runtime's rule
        // (operate::surface::choice), the one the command line reads by.
        string asked = new JsonObject
        {
            ["base"] = basePath,
            ["environment"] = HostEnvironment(),
            ["kept"] = keeper is not null,
        }.ToJsonString();

        if (!RuntimeLibrary.Rules.Surfaces.Named(
            asked, SurfaceChoice.TableOf(configuration), out string answer))
        {
            throw new InvalidOperationException(answer);
        }

        using System.Text.Json.JsonDocument named = System.Text.Json.JsonDocument.Parse(answer);
        System.Text.Json.JsonElement files = named.RootElement;
        string? File(string name) => files.GetProperty(name).GetString();

        return files.GetProperty("kept").GetBoolean() && keeper is not null
            ? Kept(File("certificate")!, File("anchors"), keeper)
            : Checked(File("certificate"), File("key"), File("anchors"));
    }

    /// <summary>The variables a host's environment names its files by, as
    /// .NET sees them, for the runtime to read in place of its process's: a
    /// variable .NET set never reaches the process's own on Unix.</summary>
    internal static JsonObject HostEnvironment()
    {
        return new JsonObject
        {
            [CertificateVariable] = Environment.GetEnvironmentVariable(CertificateVariable),
            [PrivateKeyVariable] = Environment.GetEnvironmentVariable(PrivateKeyVariable),
            [TrustAnchorVariable] = Environment.GetEnvironmentVariable(TrustAnchorVariable),
        };
    }

    /// <summary>
    /// The certificate chain at <paramref name="certificatePath"/> with the key
    /// at <paramref name="privateKeyPath"/>, and the anchors at
    /// <paramref name="trustAnchorPath"/>; all PEM, any of them null. What
    /// they hold the runtime reads, and refuses in words, as it is handed
    /// them.
    /// </summary>
    /// <exception cref="InvalidOperationException">As
    /// <see cref="From(IConfiguration, string)"/>.</exception>
    public static SecureSurface Load(
        string? certificatePath, string? privateKeyPath, string? trustAnchorPath)
    {
        // Named together or not at all, each one there: the runtime's check
        // (operate::surface::choice::named), the one the command line makes.
        string asked = new JsonObject
        {
            ["certificate"] = certificatePath,
            ["key"] = privateKeyPath,
            ["anchors"] = trustAnchorPath,
            ["kept"] = false,
        }.ToJsonString();

        return RuntimeLibrary.Rules.Surfaces.Named(asked, out string refused)
            ? Checked(certificatePath, privateKeyPath, trustAnchorPath)
            : throw new InvalidOperationException(refused);
    }

    // The files the runtime checked as it decided a surface.
    internal static SecureSurface Checked(
        string? certificatePath, string? privateKeyPath, string? trustAnchorPath)
    {
        return new SecureSurface((certificatePath, privateKeyPath, trustAnchorPath));
    }
}
