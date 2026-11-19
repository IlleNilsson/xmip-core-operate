using Xmip.Operate;

namespace Xmip.Surface;

/// <summary>A secret set through the node a listener reaches, and the secrets
/// it keeps listed, never a value (ADR-0019, amendment 2026-10-29).</summary>
public abstract partial class ListenerOperator
{
    /// <inheritdoc />
    public IReadOnlyList<SecretRecord> Secrets(string node, HeldCaller? caller)
    {
        ArgumentNullException.ThrowIfNull(node);

        return held.Node.Secrets(caller, node, out string said)
            ? SecretRecord.Listed(said)
            : throw new InvalidOperationException(said);
    }

    /// <inheritdoc />
    public SecretOperation SetSecret(
        string node, string name, string kind, string value, string who, HeldCaller? caller)
    {
        ArgumentNullException.ThrowIfNull(node);
        ArgumentNullException.ThrowIfNull(name);

        (bool applied, string result) = held.Node.SetSecret(caller, node, name, kind, value);
        return new SecretOperation(node, name, applied, result);
    }
}
