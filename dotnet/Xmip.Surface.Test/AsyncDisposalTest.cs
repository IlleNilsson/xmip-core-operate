using System.Reflection;

namespace Xmip.Surface.Test;

/// <summary>
/// xUnit v3 disposes a test class through <see cref="IAsyncDisposable"/>
/// where it has it, and through <see cref="IDisposable"/> only where it has
/// not (<c>TestRunner.DisposeTestClassInstance</c>). A bUnit page test is
/// both, so cleanup in its own <c>IDisposable.Dispose</c> never runs: until
/// 2026-11-19 the GUI's page tests removed their folders there, and about
/// 1,267 folders per label were left under the temporary directory (ADR-0021,
/// amendment 2026-11-20). One file, compiled into every .NET test project that
/// makes a <see cref="TestFolder"/> (each links it), so each holds its own
/// test classes.
/// </summary>
public sealed class AsyncDisposalTest
{
    [Fact]
    public void NoTestClassXunitDisposesAsynchronouslyCleansUpInItsOwnDispose()
    {
        string[] never =
        [
            .. typeof(AsyncDisposalTest).Assembly.GetTypes()
                .Where(type => type is { IsClass: true, IsAbstract: false })
                .Where(type => type.GetMethods().Any(method =>
                    method.GetCustomAttributes<FactAttribute>(inherit: true).Any()))
                .Where(type => typeof(IAsyncDisposable).IsAssignableFrom(type))
                .Where(type => typeof(IDisposable).IsAssignableFrom(type))
                .Where(type => type.GetInterfaceMap(typeof(IDisposable))
                    .TargetMethods.Any(method => method.DeclaringType == type))
                .Select(type => type.FullName ?? type.Name),
        ];

        Assert.Empty(never);
    }
}
