namespace Xmip.Surface;

// SecureSurface where this end's key is kept by its platform's keeper and
// never read (ADR-0034, amendment 2026-10-15): the chain and the anchors are
// files, the key the IOperatorKeeper's.
public sealed partial class SecureSurface
{
    /// <summary>
    /// The chain at <paramref name="certificatePath"/>, whose key
    /// <paramref name="keeper"/> holds and signs with, and the anchors at
    /// <paramref name="trustAnchorPath"/> (ADR-0034, amendment 2026-10-15).
    /// The chain need not be there yet — it is kept once the authority has
    /// signed (<see cref="OperatorCertificate.Keep"/>) — and is read by the
    /// runtime when a remote surface opens, which says in words what is not
    /// kept yet (<see cref="RemoteOperator"/>). What this end presents is for
    /// a node's operate listener only: a web host serves nothing with it.
    /// </summary>
    public static SecureSurface Kept(
        string certificatePath, string? trustAnchorPath, IOperatorKeeper keeper)
    {
        ArgumentNullException.ThrowIfNull(certificatePath);
        ArgumentNullException.ThrowIfNull(keeper);

        // Neither file need be there yet: both are read as a remote opens.
        return new SecureSurface((certificatePath, null, trustAnchorPath))
        {
            Keeper = keeper,
        };
    }

    /// <summary>The keeper that holds this end's key, where a configuration
    /// named a certificate and no private key for a host that has one;
    /// null where the key is a file.</summary>
    public IOperatorKeeper? Keeper { get; private init; }
}
