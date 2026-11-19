using System.Runtime.CompilerServices;
using System.Threading.Channels;
using Xmip.Operate;

namespace Xmip.Surface;

/// <summary>
/// One surface the runtime opened (section 28.2), as every .NET operator
/// over it holds it: its reading kept until the runtime says another is
/// there, and its one follow told to every watch. What a reading is, when a
/// publication is read again and what a follow tells are the runtime's
/// (<c>operate::surface</c>); this keeps what .NET holds and waits.
/// </summary>
/// <param name="node">The surface the runtime opened.</param>
/// <param name="readsAhead">Whether a telling is read before a watch is
/// told of it, and nothing asked of the runtime between tellings while one
/// follows: a snapshot's, whose follow reads each publication once as it
/// changes so that no view does (ADR-0052, amendment 2026-09-15). A node's
/// listener is told first and read by the first question after it, as the
/// runtime's follow says: until 2026-10-13 a .NET watch heard of a change
/// 1.7 ms after the node posted it.</param>
internal sealed class HeldSurface(SurfaceNode node, bool readsAhead) : IDisposable
{
    // How long one wait on the runtime's follow lasts before the follower
    // looks whether any watch is left.
    private static readonly TimeSpan Turn = TimeSpan.FromMilliseconds(200);

    private readonly Lock gate = new();
    private PublicationReading? held;
    private ulong reading = ulong.MaxValue;

    // How many tellings the watches heard, how many watches follow, and the
    // tellings heard when what is held was read.
    private long told;
    private int watching;
    private long readAt = -1;

    // The watches, each told through a queue of its own that keeps the
    // latest change; the one thread that waits on the runtime's follow for
    // them while any is told, and the tellings it heard; and the follow's
    // first telling, which every snapshot watch begins at.
    private readonly Lock watchGate = new();
    private readonly List<Channel<SurfaceChange>> watches = [];
    private readonly TaskCompletionSource begun =
        new(TaskCreationOptions.RunContinuationsAsynchronously);
    private Thread? follower;
    private ulong heard;

    public SurfaceNode Node { get; } = node;

    public string Source => Node.Source();

    /// <summary>What the surface answers from now, read again only where the
    /// runtime says another reading is there.</summary>
    public PublicationReading Read()
    {
        if (readsAhead
            && Volatile.Read(ref watching) > 0
            && Volatile.Read(ref readAt) == Volatile.Read(ref told)
            && Volatile.Read(ref held) is { } followed)
        {
            return followed;
        }

        lock (gate)
        {
            long heard = Volatile.Read(ref told);
            SurfaceReading read = Node.Read(reading);

            if (!read.Changed && held is { } same)
            {
                Volatile.Write(ref readAt, heard);
                return same;
            }

            if (read.Publication == 0 || read.Index == 0)
            {
                return held ?? PublicationReading.Nothing(Source);
            }

            PublicationReading adopted = PublicationReading.Of(read, Source);
            reading = read.Reading;
            Volatile.Write(ref held, adopted);
            Volatile.Write(ref readAt, heard);
            return adopted;
        }
    }

    /// <summary>The change the surface is at now, then one each time its
    /// follow tells, until <paramref name="stop"/>.</summary>
    public async IAsyncEnumerable<SurfaceChange> WatchAsync(
        [EnumeratorCancellation] CancellationToken stop = default)
    {
        Channel<SurfaceChange> changes = Channel.CreateBounded<SurfaceChange>(
            new BoundedChannelOptions(1)
            {
                FullMode = BoundedChannelFullMode.DropOldest,
                SingleReader = true,
                SingleWriter = true,
            });
        Watch(changes);

        try
        {
            // A snapshot's follow is started, and its first telling waited
            // for and read, before the change it is at is said, on the
            // caller's own step: the runtime watches the file's folder and
            // tells at once, so the change it is at comes as every watch's
            // first, before a view renders over it, and what is told after
            // is a change. Said on another thread, it came after the view
            // had rendered and re-rendered it beneath the operator
            // (ClusterTopologyTest, 2026-11-17). A node's follow may wait on
            // its node to answer, so its change is said first.
            if (readsAhead)
            {
                Begun(stop);
            }

            yield return SurfaceChange.Initial(Source);

            while (await Next(changes, stop).ConfigureAwait(false))
            {
                while (changes.Reader.TryRead(out SurfaceChange? change))
                {
                    yield return change;
                }
            }
        }
        finally
        {
            Unwatch(changes);
        }
    }

    /// <summary>An act on the <paramref name="noun"/> called
    /// <paramref name="target"/> of the node at <paramref name="at"/>, in the
    /// runtime's words, as <paramref name="caller"/>: what came of it.</summary>
    public (ActOutcome Outcome, string Said) Act(
        HeldCaller? caller, string at, string noun, string target, string act)
    {
        (int outcome, string said) = Node.Act(caller, at, noun, target, act);
        return (Outcome(outcome), said);
    }

    /// <summary>An act on the failed Message of <paramref name="journey"/> at
    /// <paramref name="scope"/>, asked of the node it is on.</summary>
    public MessageOperation ActOnMessage(
        HeldCaller? caller, string scope, string journey, MessageAct act)
    {
        (string at, int outcome, string said) =
            Node.ActMessage(caller, scope, journey, MessageOperation.Word(act));
        ActOutcome came = Outcome(outcome);
        return new MessageOperation(at, journey, act, came == ActOutcome.Taken, said)
        {
            Outcome = came,
        };
    }

    /// <summary>Pause or resume <paramref name="scope"/> as
    /// <paramref name="who"/>.</summary>
    public ScopeOperation Control(string scope, ScopeAction action, string who)
    {
        (bool applied, string result) = Node.Control(
            scope, action == ScopeAction.Pause ? "pause" : "resume", who);
        return new ScopeOperation(scope, action, applied, result);
    }

    /// <summary>The Messages that failed at or beneath <paramref name="scope"/>,
    /// each Port's page from <paramref name="from"/>, at most
    /// <paramref name="most"/>.</summary>
    public FailedMessageList Failed(string scope, ulong from, uint most)
    {
        return FailedMessageList.Parse(
            System.Text.Encoding.UTF8.GetBytes(Node.Failed(scope, from, most)));
    }

    public void Dispose()
    {
        Node.Dispose();
    }

    // A watch told from now, and the follower started where none waits.
    private void Watch(Channel<SurfaceChange> changes)
    {
        lock (watchGate)
        {
            watches.Add(changes);
            Interlocked.Increment(ref watching);

            if (follower is null)
            {
                follower = new Thread(Follow)
                {
                    IsBackground = true,
                    Name = "xmip-surface-watch",
                };
                follower.Start();
            }
        }
    }

    private void Unwatch(Channel<SurfaceChange> changes)
    {
        lock (watchGate)
        {
            _ = watches.Remove(changes);
            Interlocked.Decrement(ref watching);
        }
    }

    // The follow's first telling — it tells once at once, after the reading
    // it starts from — waited for, then the reading as it is now, read and
    // asked ahead: where a snapshot's watch begins.
    private void Begun(CancellationToken stop)
    {
        try
        {
            begun.Task.Wait(stop);
        }
        catch (OperationCanceledException)
        {
            return;
        }

        Interlocked.Increment(ref told);
        Read().AskAhead();
    }

    // Whether another change is there, waited for with no thread held, or
    // none once the watch is stopped or the surface released.
    private static async ValueTask<bool> Next(
        Channel<SurfaceChange> changes, CancellationToken stop)
    {
        try
        {
            return await changes.Reader.WaitToReadAsync(stop).ConfigureAwait(false);
        }
        catch (OperationCanceledException)
        {
            // The normal end of a watch.
            return false;
        }
    }

    // The one wait on the runtime's follow, on a thread of its own while any
    // watch is told, so no watch holds a thread of the pool between
    // tellings: each telling read, and what every view asks of it asked,
    // where this reads ahead, then told to every watch at once. The follow's
    // first telling is the change each watch begins at, never told as one.
    // A surface released, or a wait the runtime fails, ends every watch.
    private void Follow()
    {
        try
        {
            while (true)
            {
                ulong from = heard;
                (ulong tells, ulong revision) = Node.Wait(from, Turn);

                if (Node.Disposed)
                {
                    End(null);
                    return;
                }

                if (tells > from)
                {
                    heard = from == 0 ? 1 : tells;

                    if (readsAhead)
                    {
                        Interlocked.Increment(ref told);
                        Read().AskAhead();
                    }

                    if (from == 0)
                    {
                        _ = begun.TrySetResult();
                    }
                    else
                    {
                        Tell(new SurfaceChange(
                            revision, SurfaceChangeKind.All, DateTimeOffset.UtcNow, Source));
                    }
                }

                lock (watchGate)
                {
                    if (watches.Count == 0)
                    {
                        follower = null;
                        return;
                    }
                }
            }
        }
        catch (Exception failed) when (failed is not OutOfMemoryException)
        {
            End(failed);
        }
    }

    private void Tell(SurfaceChange change)
    {
        lock (watchGate)
        {
            foreach (Channel<SurfaceChange> watch in watches)
            {
                _ = watch.Writer.TryWrite(change);
            }
        }
    }

    // Every watch ended, with what failed where the wait failed.
    private void End(Exception? failed)
    {
        lock (watchGate)
        {
            foreach (Channel<SurfaceChange> watch in watches)
            {
                _ = watch.Writer.TryComplete(failed);
            }

            follower = null;
        }

        _ = begun.TrySetResult();
    }

    // XMIP_ACTED_*, as section 28.3 numbers what came of an act.
    private static ActOutcome Outcome(int outcome)
    {
        return outcome switch
        {
            0 => ActOutcome.Taken,
            1 => ActOutcome.Asked,
            _ => ActOutcome.Refused,
        };
    }
}
