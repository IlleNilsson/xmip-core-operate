using Xmip.Abi.Operate;

namespace Xmip.Operate;

/// <summary>
/// One scope as a publication's scope tree holds it (section 28.1,
/// <c>XmipScopePlaced</c>), copied out of the index: its records by their
/// places in what the tree was built from, worst first, the record
/// published at it, its moods, its stage and its six figures, each null where
/// none was counted.
/// </summary>
/// <param name="Scope">The scope as the tree spells it.</param>
/// <param name="Label">The segment it is called by.</param>
/// <param name="Leaves">The places of the records at and beneath it, worst
/// first.</param>
/// <param name="Own">The place of the record published at it itself, or
/// -1.</param>
/// <param name="Mood">A leaf's own mood, a parent's rollup; null where
/// nothing is recorded.</param>
/// <param name="RowMood">The rollup where records lie beneath it, else its
/// worst record's mood: what its row says.</param>
/// <param name="IsLeaf">Whether nothing is beneath it in the tree.</param>
/// <param name="IsContainer">Whether records lie beneath it, not only at
/// it.</param>
/// <param name="BeginsStage">Whether a stage of the message path begins
/// at it.</param>
/// <param name="Stage">Its stage's word; empty on none.</param>
/// <param name="Sums">Its six figures in <see cref="Counted"/> order,
/// Streams to Failed.</param>
public sealed record ScopePlacement(
    string Scope,
    string Label,
    int[] Leaves,
    long Own,
    HealthState? Mood,
    HealthState? RowMood,
    bool IsLeaf,
    bool IsContainer,
    bool BeginsStage,
    string Stage,
    ulong?[] Sums);
