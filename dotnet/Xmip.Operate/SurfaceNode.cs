using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;
using Xmip.Abi.Module;

namespace Xmip.Operate;

/// <summary>What a surface read, where it was another than the one asked
/// after: which reading it is, its revision, where a drill starts, who
/// published in its own words, and — where it changed — the publication as
/// section 8 lends one and its scope tree as section 28.1 holds one, each the
/// caller's to free; zero otherwise.</summary>
public sealed record SurfaceReading(
    ulong Reading, bool Changed, ulong Revision, string Root, string Said,
    nint Publication, nint Index);

/// <summary>
/// One surface the runtime opened (section 28.2): read, followed, asked to
/// act. Any thread may call it at once; disposing it waits for every call in
/// flight, then releases the surface and ends its follow.
/// </summary>
public sealed class SurfaceNode : IDisposable
{
    private readonly SurfaceCalls calls;
    private readonly nint handle;
    // The keeper a kept surface signs with, held until the surface is freed.
    private readonly GCHandle kept;
    private int inFlight;
    private int disposed;

    internal SurfaceNode(SurfaceCalls calls, nint handle, GCHandle kept = default)
    {
        this.calls = calls;
        this.handle = handle;
        this.kept = kept;
    }

    /// <summary>Whether it was released: nothing it is asked crosses then.</summary>
    public bool Disposed => Volatile.Read(ref disposed) == 1;

    /// <summary>Where its answers come from, in words.</summary>
    public string Source()
    {
        if (!Enter())
        {
            return "disposed";
        }

        try
        {
            _ = calls.Source(handle, out string said);
            return said;
        }
        finally
        {
            Exit();
        }
    }

    /// <summary>How it stands, nothing asked of a node: whether its node
    /// answered when last asked — a snapshot, whether its file is there — why
    /// not, and whether it follows the cluster's view its node gathers.</summary>
    public (bool Connected, string Reason, bool Gathers) Standing()
    {
        JsonElement said = Asked(() => calls.Standing(handle, out string text) == 0
            ? text
            : throw Refused(text));
        return (
            said.GetProperty("connected").GetBoolean(),
            Text(said, "reason"),
            said.GetProperty("gathers").GetBoolean());
    }

    /// <summary>Whether it answers now: a node that answers, a snapshot whose
    /// file is there.</summary>
    public bool Answers()
    {
        if (!Enter())
        {
            return false;
        }

        try
        {
            byte answers = 0;

            unsafe
            {
                Check(calls.Answers(handle, &answers));
            }

            return answers != 0;
        }
        finally
        {
            Exit();
        }
    }

    /// <summary>What it answers from now; the publication and its tree handed
    /// only where the reading is another than <paramref name="seen"/>.</summary>
    public SurfaceReading Read(ulong seen)
    {
        if (!Enter())
        {
            return new SurfaceReading(seen, false, 0, "xmip:///", string.Empty, 0, 0);
        }

        try
        {
            // Asked first with no room: an unchanged reading answers nothing
            // and costs no allocation; a changed one says how much room its
            // head needs.
            byte[] text = [];

            while (true)
            {
                nint publication = 0;
                nint index = 0;
                nuint length = 0;
                int status;

                fixed (byte* written = text)
                {
                    unsafe
                    {
                        status = calls.Read(
                            handle, seen, &publication, &index, written, (nuint)text.Length,
                            &length);
                    }
                }

                if (status == (int)XmipStatus.Truncated)
                {
                    text = new byte[(int)length];
                    continue;
                }

                Check(status);

                if (length == 0)
                {
                    return new SurfaceReading(seen, false, 0, "xmip:///", string.Empty, 0, 0);
                }

                using JsonDocument head = JsonDocument.Parse(text.AsMemory(0, (int)length));
                JsonElement said = head.RootElement;
                return new SurfaceReading(
                    said.GetProperty("reading").GetUInt64(),
                    said.GetProperty("changed").GetBoolean(),
                    said.GetProperty("revision").GetUInt64(),
                    said.GetProperty("root").GetString() ?? "xmip:///",
                    said.GetProperty("said").GetString() ?? string.Empty,
                    publication,
                    index);
            }
        }
        finally
        {
            Exit();
        }
    }

    /// <summary>Wait until its follow has told more than <paramref name="seen"/>
    /// times, or <paramref name="within"/> passed: how many times and the
    /// revision it last told. The first wait starts the follow, which tells
    /// once at once.</summary>
    public (ulong Told, ulong Revision) Wait(ulong seen, TimeSpan within)
    {
        if (!Enter())
        {
            return (seen, 0);
        }

        try
        {
            ulong told = 0;
            ulong revision = 0;
            uint waited = (uint)Math.Clamp(within.TotalMilliseconds, 0, uint.MaxValue);

            unsafe
            {
                Check(calls.Wait(handle, seen, waited, &told, &revision));
            }

            return (told, revision);
        }
        finally
        {
            Exit();
        }
    }

    /// <summary>An act on the <paramref name="noun"/> called
    /// <paramref name="target"/> of the node at <paramref name="node"/>, in
    /// the runtime's words, as <paramref name="caller"/>: what came of it,
    /// XMIP_ACTED_*, and the node's sentence.</summary>
    public (int Outcome, string Said) Act(
        HeldCaller? caller, string node, string noun, string target, string act)
    {
        string asked = new JsonObject
        {
            ["node"] = node,
            ["noun"] = noun,
            ["target"] = target,
            ["act"] = act,
        }.ToJsonString();
        JsonElement said = Asked(() => calls.Act(
            handle, caller?.Handle ?? 0, asked, out string text) == 0 ? text : throw Refused(text));
        return (said.GetProperty("outcome").GetInt32(), Text(said, "said"));
    }

    /// <summary>An act on the failed Message of <paramref name="journey"/> at
    /// <paramref name="scope"/>, asked of the node the scope is on, as
    /// <paramref name="caller"/>: the node, the outcome and the sentence.</summary>
    public (string Node, int Outcome, string Said) ActMessage(
        HeldCaller? caller, string scope, string journey, string act)
    {
        string asked = new JsonObject
        {
            ["scope"] = scope,
            ["journey"] = journey,
            ["act"] = act,
        }.ToJsonString();
        JsonElement said = Asked(() => calls.ActMessage(
            handle, caller?.Handle ?? 0, asked, out string text) == 0 ? text : throw Refused(text));
        return (Text(said, "node"), said.GetProperty("outcome").GetInt32(), Text(said, "said"));
    }

    /// <summary>Pause or resume <paramref name="scope"/>, the action
    /// <c>pause</c> or <c>resume</c>, as <paramref name="who"/>.</summary>
    public (bool Applied, string Result) Control(string scope, string action, string who)
    {
        string asked = new JsonObject
        {
            ["scope"] = scope,
            ["action"] = action,
            ["who"] = who,
        }.ToJsonString();
        JsonElement said = Asked(() => calls.Control(handle, asked, out string text) == 0
            ? text
            : throw Refused(text));
        return (said.GetProperty("applied").GetBoolean(), Text(said, "result"));
    }

    /// <summary>The Messages that failed at or beneath <paramref name="scope"/>,
    /// each Port's page from <paramref name="from"/>, at most
    /// <paramref name="most"/>: section 16's JSON with <c>listed</c> and
    /// <c>failure</c>.</summary>
    public string Failed(string scope, ulong from, uint most)
    {
        string asked = new JsonObject
        {
            ["scope"] = scope,
            ["from"] = from,
            ["most"] = most,
        }.ToJsonString();
        return Within(() => calls.Failed(handle, asked, out string text) == 0
            ? text
            : throw Refused(text));
    }

    /// <summary>The secrets the node at <paramref name="node"/> keeps, as
    /// <paramref name="caller"/>: true with their JSON, false with why not.</summary>
    public bool Secrets(HeldCaller? caller, string node, out string said)
    {
        string text = string.Empty;
        int status = 0;
        _ = Within(() =>
        {
            status = calls.Secrets(handle, caller?.Handle ?? 0, node, out text);
            return text;
        });
        said = text;
        return status == 0;
    }

    /// <summary>Set the secret <paramref name="name"/> of <paramref name="kind"/>
    /// to <paramref name="value"/> through the node at <paramref name="node"/>,
    /// as <paramref name="caller"/>: whether it was set, and the sentence.</summary>
    public (bool Applied, string Result) SetSecret(
        HeldCaller? caller, string node, string name, string kind, string value)
    {
        string asked = new JsonObject
        {
            ["node"] = node,
            ["name"] = name,
            ["kind"] = kind,
        }.ToJsonString();
        JsonElement said = Asked(() => calls.SetSecret(
            handle, caller?.Handle ?? 0, asked, value, out string text) == 0
            ? text
            : throw Refused(text));
        return (said.GetProperty("applied").GetBoolean(), Text(said, "result"));
    }

    /// <summary>A surface nobody disposed is released when it is collected.</summary>
    ~SurfaceNode()
    {
        Release();
    }

    /// <summary>Release the surface once every call in flight returned.</summary>
    public void Dispose()
    {
        Release();
        GC.SuppressFinalize(this);
    }

    private void Release()
    {
        if (Interlocked.Exchange(ref disposed, 1) == 1)
        {
            return;
        }

        SpinWait spin = default;

        while (Volatile.Read(ref inFlight) > 0)
        {
            spin.SpinOnce();
        }

        unsafe
        {
            calls.Free(handle);
        }

        if (kept.IsAllocated)
        {
            unsafe
            {
                kept.Free();
            }
        }
    }

    private JsonElement Asked(Func<string> ask)
    {
        string text = Within(ask);
        using JsonDocument answer = JsonDocument.Parse(Encoding.UTF8.GetBytes(text));
        return answer.RootElement.Clone();
    }

    private string Within(Func<string> ask)
    {
        if (!Enter())
        {
            throw new ObjectDisposedException(nameof(SurfaceNode));
        }

        try
        {
            return ask();
        }
        finally
        {
            Exit();
        }
    }

    private static string Text(JsonElement said, string name)
    {
        return said.GetProperty(name).GetString() ?? string.Empty;
    }

    private static InvalidOperationException Refused(string text)
    {
        return new InvalidOperationException(text);
    }

    private static void Check(int status)
    {
        if (status != 0)
        {
            throw new InvalidOperationException(
                $"the runtime answered a surface with {(XmipStatus)status}");
        }
    }

    private bool Enter()
    {
        Interlocked.Increment(ref inFlight);

        if (Volatile.Read(ref disposed) == 0)
        {
            return true;
        }

        Interlocked.Decrement(ref inFlight);
        return false;
    }

    private void Exit()
    {
        Interlocked.Decrement(ref inFlight);
    }
}
