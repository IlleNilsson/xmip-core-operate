using System.Collections.Concurrent;
using System.Runtime.CompilerServices;
using System.Text.Json;

namespace Xmip.Surface;

/// <summary>
/// A node's nouns listed, chosen and ordered as a query asks, by the one
/// rule every surface lists them by: <c>operate::surface</c>'s queries over
/// <c>list_order</c> — a text compared ordinally — asked through
/// <c>xmip_list_query_v1</c> (ADR-0052, amendment 2026-11-18). The list
/// crosses as its section's JSON and comes back as places in it, so every
/// record a surface read is the one it shows.
/// </summary>
internal static class ListQuery
{
    // What a list that does not change was asked, by the question, kept as
    // long as the list: a publication's list is read once and asked again by
    // every render over it, so each question crosses once a publication.
    private static readonly ConditionalWeakTable<object, ConcurrentDictionary<string, int[]>>
        Answered = [];

    /// <summary>
    /// The entries of <paramref name="entries"/> that <paramref name="query"/>
    /// chooses, in its order: the list <paramref name="list"/> written under
    /// <paramref name="key"/> by <paramref name="write"/>, each entry an
    /// object with the members the query reads. A list that cannot change —
    /// a publication's — is asked each question once.
    /// </summary>
    /// <exception cref="ArgumentException">REFUSED: a sort that is no column or
    /// an order that is neither word.</exception>
    public static IReadOnlyList<T> Chosen<T>(
        int list,
        IEnumerable<T> entries,
        string key,
        Action<Utf8JsonWriter, T> write,
        Action<Utf8JsonWriter> query)
    {
        T[] given = [.. entries];
        string asked = $"{list}{JsonText.Document(query)}";
        int[] places = entries is IReadOnlyList<T> and not ICollection<T> { IsReadOnly: false }
            ? Answered.GetOrCreateValue(entries)
                .GetOrAdd(asked, _ => Placed(list, given, key, write, query))
            : Placed(list, given, key, write, query);

        return [.. places.Select(at => given[at])];
    }

    // The places the runtime chose of the list, in its order.
    private static int[] Placed<T>(
        int list,
        T[] given,
        string key,
        Action<Utf8JsonWriter, T> write,
        Action<Utf8JsonWriter> query)
    {
        using JsonDocument answer = Asked(list, Listed(given, key, write), query);
        return [.. Places(answer.RootElement, "places")];
    }

    /// <summary>The list written as its section's JSON: <c>{"key": [...]}</c>.</summary>
    public static string Listed<T>(
        IReadOnlyList<T> entries, string key, Action<Utf8JsonWriter, T> write)
    {
        return JsonText.Document(writer =>
        {
            writer.WriteStartArray(key);
            foreach (T entry in entries)
            {
                writer.WriteStartObject();
                write(writer, entry);
                writer.WriteEndObject();
            }

            writer.WriteEndArray();
        });
    }

    /// <summary>The runtime's answer to <paramref name="query"/> over
    /// <paramref name="listed"/>.</summary>
    public static JsonDocument Asked(int list, string listed, Action<Utf8JsonWriter> query)
    {
        return RuntimeLibrary.Rules.Sentences.ListQuery(list, listed, JsonText.Document(query));
    }

    /// <summary>The places an answer lists under <paramref name="name"/>.</summary>
    public static IEnumerable<int> Places(JsonElement answer, string name)
    {
        return answer.GetProperty(name).EnumerateArray().Select(at => at.GetInt32());
    }

    /// <summary>A moment as the section's JSON writes one: Unix nanoseconds.</summary>
    public static long Nanos(DateTimeOffset at)
    {
        return (at - DateTimeOffset.UnixEpoch).Ticks * 100;
    }

    /// <summary>A text member, where it is given.</summary>
    public static void Text(Utf8JsonWriter writer, string name, string? value)
    {
        if (value is not null)
        {
            writer.WriteString(name, value);
        }
    }

    /// <summary>The words a row shows (<c>xmip_list_words_v1</c>).</summary>
    public static string Word(int which, string first, string second)
    {
        return RuntimeLibrary.Rules.Sentences.ListWord(which, first, second);
    }
}
