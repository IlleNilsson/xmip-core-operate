using System.Collections;
using Xmip.Operate;

namespace Xmip.Surface;

/// <summary>
/// The records at and beneath a scope, as a <see cref="ScopeIndex"/> answers
/// them: their places in the records the index was built from, read through
/// to those records, so a scope's list is never a copy of them.
/// </summary>
internal sealed class PlacedRecords(IReadOnlyList<HealthRecord> records, int[] places)
    : IReadOnlyList<HealthRecord>
{
    public HealthRecord this[int index] => records[places[index]];

    public int Count => places.Length;

    public IEnumerator<HealthRecord> GetEnumerator()
    {
        foreach (int at in places)
        {
            yield return records[at];
        }
    }

    IEnumerator IEnumerable.GetEnumerator()
    {
        return GetEnumerator();
    }
}
