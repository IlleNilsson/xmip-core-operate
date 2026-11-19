using System.Runtime.InteropServices;
using System.Text.Json.Nodes;
using Xmip.Abi.Module;
using Xmip.Abi.Operate;

namespace Xmip.Operate;

// Sections 19 and 23 as a surface asks them of files and names: a person's
// request signed into its chain's file, and the name a cluster's own
// authority is made under, each the rule the command line calls
// (xmip_operator_sign_file_v1, xmip_authority_name_v1; ADR-0052, amendment
// 2026-11-17). A node's build only, bound on first asking.
public sealed partial class RuntimeCertificates
{
    private delegate* unmanaged[Cdecl]<XmipStr, XmipStr, byte*, nuint, nuint*, int> _signFile;
    private delegate* unmanaged[Cdecl]<XmipStr, XmipStr, byte*, nuint, nuint*, int> _named;

    /// <summary>The symbols only a node's build exports, beside
    /// <see cref="NodeEntrypoints"/>, bound where they are first asked.</summary>
    public static IReadOnlyList<string> WrittenEntrypoints { get; } =
        [OperateAbi.OperatorSignFileEntrypoint, OperateAbi.AuthorityNameEntrypoint];

    /// <summary>
    /// The request at <paramref name="request"/> signed by the authority the
    /// node configuration at <paramref name="configuration"/> names, its
    /// chain kept whole at <paramref name="keptAt"/> where one is named and
    /// never over a file: the chain, PEM, leaf first; or false with the
    /// runtime's sentence in <paramref name="refused"/>.
    /// </summary>
    /// <exception cref="InvalidOperationException">The library is a remote
    /// build, which signs nothing.</exception>
    public bool SignFile(
        string request, string configuration, string? keptAt, out string chain,
        out string refused)
    {
        ArgumentNullException.ThrowIfNull(request);
        ArgumentNullException.ThrowIfNull(configuration);

        string argument = new JsonObject
        {
            ["configuration"] = configuration,
            ["chain"] = keptAt,
        }.ToJsonString();
        bool signed = RuntimeDesign.Call(
            Bound(ref _signFile, OperateAbi.OperatorSignFileEntrypoint),
            request,
            argument,
            out string answer);

        chain = signed ? answer : string.Empty;
        refused = signed ? string.Empty : answer;
        return signed;
    }

    /// <summary>The name the cluster <paramref name="cluster"/> makes its own
    /// authority under.</summary>
    /// <exception cref="InvalidOperationException">The library is a remote
    /// build, which keeps no authority.</exception>
    public string AuthorityName(string cluster)
    {
        ArgumentNullException.ThrowIfNull(cluster);

        _ = RuntimeDesign.Call(
            Bound(ref _named, OperateAbi.AuthorityNameEntrypoint),
            cluster,
            string.Empty,
            out string name);
        return name;
    }

    // An export of a node's build, bound once; a remote build refused in its
    // words.
    private delegate* unmanaged[Cdecl]<XmipStr, XmipStr, byte*, nuint, nuint*, int> Bound(
        ref delegate* unmanaged[Cdecl]<XmipStr, XmipStr, byte*, nuint, nuint*, int> held,
        string entrypoint)
    {
        if (held is null)
        {
            held = NativeLibrary.TryGetExport(_library, entrypoint, out nint found)
                ? (delegate* unmanaged[Cdecl]<XmipStr, XmipStr, byte*, nuint, nuint*, int>)found
                : throw RemoteBuild.Refused(_library, WrittenEntrypoints);
        }

        return held;
    }
}
