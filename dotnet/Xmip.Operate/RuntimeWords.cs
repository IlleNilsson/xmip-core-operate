using System.Text;
using Xmip.Abi.Module;
using Xmip.Abi.Operate;

namespace Xmip.Operate;

/// <summary>Section 28.3 of <c>include/xmip_operate.h</c>: what a surface
/// says and how it orders a list, each word written once in Rust and asked
/// for here by every .NET surface (ADR-0052, amendment 2026-11-18). The fixed
/// words — the phrases, the act words, the outcomes' words and each list's
/// columns — are read once, when the library loads; a sentence is asked for
/// when it is said.</summary>
public sealed partial class RuntimeWords
{
    // Room for a figure or a sentence on the stack; a longer answer is
    // asked again at its length.
    private const int Room = 256;

    private readonly delegate* unmanaged[Cdecl]<int, XmipStr*, int> _phrase;
    private readonly delegate* unmanaged[Cdecl]<int, XmipStr*, int> _actWord;
    private readonly delegate* unmanaged[Cdecl]<int, int*, XmipStr*, int> _actOutcome;
    private readonly delegate* unmanaged[Cdecl]<int, XmipStatusMeaning*, int> _statusMeaning;
    private readonly delegate* unmanaged[Cdecl]<XmipBoundaries*, int> _boundaries;
    private readonly delegate* unmanaged[Cdecl]<
        int, XmipStr, XmipStr, byte*, nuint, nuint*, int> _sentence;
    private readonly delegate* unmanaged[Cdecl]<int, XmipStr, byte*, nuint, nuint*, int> _actSaid;
    private readonly delegate* unmanaged[Cdecl]<long, long, byte*, nuint, nuint*, int> _age;
    private readonly delegate* unmanaged[Cdecl]<byte, ulong, byte*, nuint, nuint*, int> _figure;
    private readonly delegate* unmanaged[Cdecl]<
        byte*, ulong*, byte*, nuint, nuint*, int> _figuresLine;
    private readonly delegate* unmanaged[Cdecl]<int, XmipStr, byte*, nuint, nuint*, int> _text;
    private readonly delegate* unmanaged[Cdecl]<XmipStr, XmipStr, byte*, nuint, nuint*, int> _who;
    private readonly delegate* unmanaged[Cdecl]<
        XmipStr, int, XmipStr*, nuint, byte*, nuint, nuint*, int> _validated;
    private readonly delegate* unmanaged[Cdecl]<
        XmipStr, XmipStr, byte*, nuint, nuint*, int> _runSaid;
    private readonly delegate* unmanaged[Cdecl]<
        XmipStr, XmipStr, byte*, nuint, nuint*, int> _capabilitySaid;
    private readonly delegate* unmanaged[Cdecl]<
        int, XmipStr, XmipStr, byte*, nuint, nuint*, int> _listQuery;
    private readonly delegate* unmanaged[Cdecl]<int, XmipStr*, nuint, nuint*, int> _listColumns;
    private readonly delegate* unmanaged[Cdecl]<
        int, XmipStr, XmipStr, byte*, nuint, nuint*, int> _listWords;

    private readonly string[] _phrases;
    private readonly string[] _actWords;
    private readonly string[] _outcomeWords;
    private readonly string[][] _columns;

    internal RuntimeWords(nint library)
    {
        _phrase = (delegate* unmanaged[Cdecl]<int, XmipStr*, int>)Export(
            library, OperateAbi.PhraseEntrypoint);
        _actWord = (delegate* unmanaged[Cdecl]<int, XmipStr*, int>)Export(
            library, OperateAbi.ActWordEntrypoint);
        _actOutcome = (delegate* unmanaged[Cdecl]<int, int*, XmipStr*, int>)Export(
            library, OperateAbi.ActOutcomeEntrypoint);
        _statusMeaning = (delegate* unmanaged[Cdecl]<int, XmipStatusMeaning*, int>)Export(
            library, OperateAbi.StatusMeaningEntrypoint);
        _boundaries = (delegate* unmanaged[Cdecl]<XmipBoundaries*, int>)Export(
            library, OperateAbi.BoundariesEntrypoint);
        _sentence = (delegate* unmanaged[Cdecl]<int, XmipStr, XmipStr, byte*, nuint, nuint*, int>)
            Export(library, OperateAbi.SentenceEntrypoint);
        _actSaid = (delegate* unmanaged[Cdecl]<int, XmipStr, byte*, nuint, nuint*, int>)Export(
            library, OperateAbi.ActSaidEntrypoint);
        _age = (delegate* unmanaged[Cdecl]<long, long, byte*, nuint, nuint*, int>)Export(
            library, OperateAbi.AgeEntrypoint);
        _figure = (delegate* unmanaged[Cdecl]<byte, ulong, byte*, nuint, nuint*, int>)Export(
            library, OperateAbi.FigureEntrypoint);
        _figuresLine = (delegate* unmanaged[Cdecl]<byte*, ulong*, byte*, nuint, nuint*, int>)
            Export(library, OperateAbi.FiguresLineEntrypoint);
        _text = (delegate* unmanaged[Cdecl]<int, XmipStr, byte*, nuint, nuint*, int>)Export(
            library, OperateAbi.TextEntrypoint);
        _who = RuntimeDesign.Export(library, OperateAbi.WhoEntrypoint);
        _validated = (delegate* unmanaged[Cdecl]<
            XmipStr, int, XmipStr*, nuint, byte*, nuint, nuint*, int>)Export(
            library, OperateAbi.ValidatedEntrypoint);
        _runSaid = RuntimeDesign.Export(library, OperateAbi.RunSaidEntrypoint);
        _capabilitySaid = RuntimeDesign.Export(library, OperateAbi.CapabilitySaidEntrypoint);
        _listQuery = (delegate* unmanaged[Cdecl]<int, XmipStr, XmipStr, byte*, nuint, nuint*, int>)
            Export(library, OperateAbi.ListQueryEntrypoint);
        _listColumns = (delegate* unmanaged[Cdecl]<int, XmipStr*, nuint, nuint*, int>)Export(
            library, OperateAbi.ListColumnsEntrypoint);
        _listWords = (delegate* unmanaged[Cdecl]<int, XmipStr, XmipStr, byte*, nuint, nuint*, int>)
            Export(library, OperateAbi.ListWordsEntrypoint);

        _phrases = [.. Enumerable.Range(0, 3).Select(at => Word(_phrase, at))];
        _actWords = [.. Enumerable.Range(0, 6).Select(at => Word(_actWord, at))];
        _outcomeWords =
        [
            ActOutcome((int)XmipStatus.Ok).Word,
            ActOutcome((int)XmipStatus.Again).Word,
            ActOutcome((int)XmipStatus.Invalid).Word,
        ];
        _columns = [.. Enumerable.Range(0, 3).Select(Listed)];
        Current = this;
    }

    /// <summary>The words of the library this process loaded last: what a
    /// binding with no rules of its own at hand — a status's meaning in a
    /// refusal — says by.</summary>
    internal static RuntimeWords? Current { get; private set; }

    /// <summary>The symbols it binds, each of which a runtime exports, its
    /// remote build too.</summary>
    public static IReadOnlyList<string> Entrypoints { get; } =
    [
        OperateAbi.PhraseEntrypoint,
        OperateAbi.ActWordEntrypoint,
        OperateAbi.ActOutcomeEntrypoint,
        OperateAbi.StatusMeaningEntrypoint,
        OperateAbi.BoundariesEntrypoint,
        OperateAbi.SentenceEntrypoint,
        OperateAbi.ActSaidEntrypoint,
        OperateAbi.AgeEntrypoint,
        OperateAbi.FigureEntrypoint,
        OperateAbi.FiguresLineEntrypoint,
        OperateAbi.TextEntrypoint,
        OperateAbi.WhoEntrypoint,
        OperateAbi.ValidatedEntrypoint,
        OperateAbi.RunSaidEntrypoint,
        OperateAbi.CapabilitySaidEntrypoint,
        OperateAbi.ListQueryEntrypoint,
        OperateAbi.ListColumnsEntrypoint,
        OperateAbi.ListWordsEntrypoint,
    ];

    /// <summary>The phrase <paramref name="which"/> names
    /// (<see cref="OperateAbi.Phrase"/>).</summary>
    public string Phrase(int which)
    {
        return _phrases[which];
    }

    /// <summary>The word the act <paramref name="act"/>
    /// (<see cref="OperateAbi.Act"/>) crosses to a node as.</summary>
    public string ActWord(int act)
    {
        return _actWords[act];
    }

    /// <summary>The word of the outcome <paramref name="outcome"/>
    /// (<see cref="OperateAbi.Outcome"/>): <c>taken</c>, <c>asked</c>,
    /// <c>refused</c>.</summary>
    public string OutcomeWord(int outcome)
    {
        return _outcomeWords[outcome];
    }

    /// <summary>The columns the list <paramref name="list"/>
    /// (<see cref="OperateAbi.List"/>) sorts by, its first the default.</summary>
    public IReadOnlyList<string> Columns(int list)
    {
        return _columns[list];
    }

    /// <summary>What came of an act whose export answered
    /// <paramref name="status"/> (<see cref="OperateAbi.Outcome"/>), and its
    /// word.</summary>
    public (int Outcome, string Word) ActOutcome(int status)
    {
        int outcome = 0;
        XmipStr word = default;
        unsafe
        {
            Check(_actOutcome(status, &outcome, &word));
            return (outcome, word.Read());
        }
    }

    /// <summary>What <paramref name="code"/> means.</summary>
    public StatusMeaning Meaning(int code)
    {
        XmipStatusMeaning said = default;
        unsafe
        {
            Check(_statusMeaning(code, &said));
            return new StatusMeaning(
                code, said.Known != 0, said.Name.Read(), said.Meaning.Read(),
                said.Retryable != 0, said.Terminal != 0);
        }
    }

    /// <summary>One line an operator can read for <paramref name="code"/>, by
    /// the library loaded last; the code's number where none is.</summary>
    internal static string Explain(int code)
    {
        return Current?.Meaning(code).Meaning ?? $"status {code}";
    }

    /// <summary>The two boundaries this build speaks.</summary>
    public AbiBoundaries Boundaries()
    {
        XmipBoundaries said = default;
        unsafe
        {
            Check(_boundaries(&said));
            return new AbiBoundaries(
                said.ModuleVersion, said.ModuleEntrypoint.Read(),
                said.ModuleLibraryFileName.Read(), said.OperateVersion,
                said.OperateEntrypoint.Read());
        }
    }

    private delegate int Writing(byte* into, nuint cap, nuint* length);

    // One number and one text, answered in text.
    private static string One(
        delegate* unmanaged[Cdecl]<int, XmipStr, byte*, nuint, nuint*, int> export,
        int which,
        string text)
    {
        Utf8Pack pack = new([text]);
        unsafe
        {
            fixed (byte* data = pack.Bytes)
            {
                XmipStr given = pack.Borrow(data, 0);
                return Said((into, cap, length) => export(which, given, into, cap, length));
            }
        }
    }

    // One number and two texts, answered in text.
    private static string Two(
        delegate* unmanaged[Cdecl]<int, XmipStr, XmipStr, byte*, nuint, nuint*, int> export,
        int which,
        string first,
        string second)
    {
        Utf8Pack pack = new([first, second]);
        unsafe
        {
            fixed (byte* data = pack.Bytes)
            {
                XmipStr one = pack.Borrow(data, 0);
                XmipStr two = pack.Borrow(data, 1);
                return Said((into, cap, length) => export(which, one, two, into, cap, length));
            }
        }
    }

    // An answer written into room on the stack, asked again at its length
    // where it did not fit.
    private static string Said(Writing write, bool check = true)
    {
        Span<byte> room = stackalloc byte[Room];
        nuint needed = 0;
        int status;
        unsafe
        {
            fixed (byte* into = room)
            {
                status = write(into, Room, &needed);
            }
        }

        if (needed > Room)
        {
            byte[] more = new byte[(int)needed];
            unsafe
            {
                fixed (byte* into = more)
                {
                    status = write(into, needed, &needed);
                }
            }

            room = more;
        }

        if (check)
        {
            Check(status);
        }

        return Encoding.UTF8.GetString(room[..(int)needed]);
    }

    private static string Answered(
        delegate* unmanaged[Cdecl]<XmipStr, XmipStr, byte*, nuint, nuint*, int> export,
        string input,
        string argument)
    {
        return RuntimeDesign.Call(export, input, argument, out string answer)
            ? answer
            : throw new ArgumentException(answer);
    }

    private static string Word(delegate* unmanaged[Cdecl]<int, XmipStr*, int> export, int which)
    {
        XmipStr word = default;
        unsafe
        {
            Check(export(which, &word));
            return word.Read();
        }
    }

    private string[] Listed(int list)
    {
        XmipStr[] columns = new XmipStr[16];
        nuint count = 0;
        unsafe
        {
            fixed (XmipStr* into = columns)
            {
                Check(_listColumns(list, into, 16, &count));
            }

            return [.. columns.Take((int)count).Select(column => column.Read())];
        }
    }

    private static long Nanos(DateTimeOffset at)
    {
        return (at - DateTimeOffset.UnixEpoch).Ticks * 100;
    }

    private static nint Export(nint library, string entrypoint)
    {
        return System.Runtime.InteropServices.NativeLibrary.GetExport(library, entrypoint);
    }

    private static void Check(int status)
    {
        if (status != (int)XmipStatus.Ok)
        {
            throw new InvalidOperationException(
                $"the runtime answered a word with status {status}");
        }
    }
}
