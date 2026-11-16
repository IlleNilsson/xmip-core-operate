using System.Reflection;
using System.Text.RegularExpressions;
using Xmip.Abi.Test;

namespace Xmip.Operate.Test;

/// <summary>
/// <see cref="RuntimeEvents"/> calls a subscribe and a listen in the shape
/// <c>xmip_operate.h</c> declares them, and neither names a Party.
/// </summary>
public sealed class EventSubscribeShapeTest
{
    [Theory]
    [InlineData("XmipEventSubscribeFn", "_subscribe")]
    [InlineData("XmipEventListenFn", "_listen")]
    public void ASubscribeNamesNoPartyAndIsBoundInTheHeadersShape(string shape, string bound)
    {
        // A subscriber is the caller the gate proved, never a Party the
        // program names (ADR-0019, amendment 2026-11-11).
        Match declared = Regex.Match(
            Header.Text("xmip_operate.h"),
            @"\(\*" + shape + @"\)\(([^;]*)\);",
            RegexOptions.Singleline);

        Assert.True(declared.Success, $"{shape} is not in xmip_operate.h");

        string[] parameters = declared.Groups[1].Value.Split(',');
        Type called = typeof(RuntimeEvents)
            .GetField(bound, BindingFlags.Instance | BindingFlags.NonPublic)?.FieldType
            ?? throw new InvalidOperationException($"RuntimeEvents binds no {bound}");

        Assert.DoesNotContain(
            parameters, parameter => parameter.Contains("subscriber", StringComparison.Ordinal));
        Assert.Equal(parameters.Length, called.GetFunctionPointerParameterTypes().Length);
    }
}
