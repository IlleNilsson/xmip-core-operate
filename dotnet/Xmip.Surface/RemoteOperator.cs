using Xmip.Operate;

namespace Xmip.Surface;

/// <summary>
/// The surface over a node on another machine, reached at its operate
/// address over gRPC inside Xmip's mutual TLS (ADR-0014, amendment
/// 2026-10-12; ADR-0063): a <see cref="ListenerOperator"/> that presents the
/// certificate its <see cref="SecureSurface"/> names and takes the node only
/// where its certificate reaches the anchors named with it. The node takes
/// every act as the subject that certificate proves. Where the key is kept
/// by the platform's keeper (<see cref="SecureSurface.Keeper"/>), the keeper
/// signs each handshake through the runtime and the key never leaves it.
/// What is missing or no identity the runtime says in <see cref="ListenerOperator.Source"/>.
/// </summary>
/// <param name="address">The node's operate address, <c>host:port</c>.</param>
/// <param name="tls">What this side presents and trusts.</param>
public sealed class RemoteOperator(string address, SecureSurface tls)
    : ListenerOperator("REMOTE", address, Opened(address, tls))
{
    /// <summary>What every surface says when <c>--remote</c> or
    /// <c>-Remote</c> names no node, in the runtime's words; null when it
    /// names one, whose address the runtime judges as it opens it.</summary>
    public static string? Refusal(string? address)
    {
        string said = RuntimeLibrary.Rules.Surfaces.RemoteRefusal(address);
        return said.Length > 0 ? said : null;
    }

    private static SurfaceNode Opened(string address, SecureSurface tls)
    {
        ArgumentNullException.ThrowIfNull(address);
        ArgumentNullException.ThrowIfNull(tls);

        (string? certificate, string? key, string? anchors) = tls.Paths;

        // A key its platform's keeper holds: the keeper signs every
        // handshake, and the key never leaves it (ADR-0034, amendment
        // 2026-10-15).
        return tls.Keeper is { } keeper && certificate is not null
            ? RuntimeLibrary.Rules.Surfaces.RemoteKept(
                address, certificate, anchors, keeper.Name,
                message => keeper.Sign(OperatorCertificate.KeptAlias, message))
            : RuntimeLibrary.Rules.Surfaces.Remote(address, certificate, key, anchors);
    }
}
