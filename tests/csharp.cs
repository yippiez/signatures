@@CASE@@ CommentsStrings
// CommentsStrings.cs — verifies that fake declarations inside comments and
// string literals of every flavour are silently ignored.

// class FakeLineCommentClass { void FakeMethod() {} }
// interface IFakeLine { int Compute(); }
// const int FAKE_LINE_CONST = 99;

/* class FakeBlockClass { public void Go() {} } */
/*
 * Multi-line block comment containing fake signatures:
 * public struct FakeStruct { int X; }
 * public enum FakeEnum { A, B }
 * public const double FakeConst = 3.14;
 */

/** <summary>class XmlDocFake { }</summary> */

namespace Demo.CommentsStrings
{
    /// <summary>
    /// This XML doc comment must NOT produce any signatures:
    /// <code>
    /// class DocFakeClass { void DocFakeMethod() {} }
    /// interface IDocFake { string Get(); }
    /// </code>
    /// </summary>
    public class RealClass
    {
        // Ordinary string — fake decls inside must be ignored.
        private const string HelpText =
            "class NotReal { void AlsoNotReal() {} } const int Nope = 0;";

        // Verbatim string — fake decls must also be ignored.
        private static readonly string SqlQuery =
            @"SELECT class, interface FROM schema WHERE enum = 'record'
              AND struct > 0";

        // Triple-quoted (C# 11 raw) string literal with fake decls.
        private static readonly string RawLiteral = """
            class InsideRaw { public void Method() {} }
            interface IInsideRaw { void Foo(); }
            const int RAW_CONST = 42;
            """;

        // Interpolated string — the expressions are real code but the text isn't.
        private string _tag = "real";
        public string Describe(int n) =>
            $"class count: {n} interface count: {n - 1} struct count: 0";

        /// <summary>A real method — it must appear in output.</summary>
        public void RealMethod() { }

        /// <summary>Real constant.</summary>
        public const int RealConst = 7;
    }

    // Line-comment-terminated interface — only the real interface below counts.
    // interface ILineFake { void Phantom(); }

    /// <summary>Real interface.</summary>
    public interface IRealInterface
    {
        /// <summary>
        /// enum FakeInDoc { X }  — must be ignored (inside XML doc)
        /// </summary>
        void RealWork();
        int RealCompute(double value);
    }

    public enum RealEnum
    {
        First,
        Second,
        Third
    }

    public struct RealStruct
    {
        public int Value;
        public RealStruct(int value) { Value = value; }

        /* struct FakeInsideBlockComment { } */
        public override string ToString() => Value.ToString();
    }

    public static class StringHelper
    {
        public static string Repeat(string s, int times) =>
            string.Concat(Enumerable.Repeat(s, times));

        public static bool Contains(string haystack, string needle) =>
            haystack.Contains(needle, StringComparison.OrdinalIgnoreCase);
    }
}
@@CASE@@ Edge
// Edge.cs — edge cases: empty bodies, very long signatures, semicolons after
// class declarations, attributes on multiple lines, partial classes,
// expression-bodied properties, operator overloads, explicit interface
// implementation, and a deliberately unclosed brace at end of file
// (malformed but parseable).

using System;

namespace Edge
{
    // Attribute spread across several lines.
    [Obsolete(
        "Use NewThing instead",
        error: false)]
    [CLSCompliant(true)]
    public class LegacyClass
    {
        public const int Version = 1;
        public LegacyClass() { }

        [Obsolete("old")]
        public void OldMethod() { }

        public void NewMethod() { }
    }

    // Partial class — two halves that together make one type.
    public partial class PartialWidget
    {
        public const string Kind = "widget";
        public int Id { get; set; }
        partial void OnIdChanged(int newId);
    }

    public partial class PartialWidget
    {
        partial void OnIdChanged(int newId) { }
        public override string ToString() => $"Widget({Id})";
    }

    // Empty class body (valid C#).
    public class EmptyClass { }

    // Single-line struct.
    public struct Point { public int X; public int Y; }

    // Operator overloads.
    public readonly struct Vector2
    {
        public readonly float X;
        public readonly float Y;

        public Vector2(float x, float y) { X = x; Y = y; }

        public static Vector2 operator +(Vector2 a, Vector2 b) =>
            new(a.X + b.X, a.Y + b.Y);

        public static Vector2 operator -(Vector2 a, Vector2 b) =>
            new(a.X - b.X, a.Y - b.Y);

        public static Vector2 operator *(Vector2 v, float s) =>
            new(v.X * s, v.Y * s);

        public static bool operator ==(Vector2 a, Vector2 b) =>
            a.X == b.X && a.Y == b.Y;

        public static bool operator !=(Vector2 a, Vector2 b) => !(a == b);

        public override string ToString() => $"({X}, {Y})";
    }

    // Explicit interface implementation.
    public interface IPrintable
    {
        void Print();
        string Format(string template);
    }

    public class Document : IPrintable
    {
        public const string DefaultTemplate = "{content}";

        void IPrintable.Print() { Console.WriteLine(Content); }
        string IPrintable.Format(string template) => template.Replace("{content}", Content);

        public string Content { get; set; } = string.Empty;

        public void Print() { Console.Write(Content); }
    }

    // Very long method signature (wraps in real editors).
    public static class LongSignatures
    {
        public static Dictionary<string, List<Tuple<int, string, bool>>> BuildComplexMap(
            IEnumerable<string> keys,
            Func<string, int> indexer,
            Func<string, string> namer,
            Func<string, bool> validator)
        {
            return new Dictionary<string, List<Tuple<int, string, bool>>>();
        }
    }

    // Enum with explicit values and attributes.
    public enum ErrorCode
    {
        None = 0,
        NotFound = 404,
        ServerError = 500,
        [Obsolete] Legacy = 999
    }

    // Record with body containing methods.
    public record LogEntry(DateTime Timestamp, string Message, int Level)
    {
        public static readonly LogEntry Empty = new(DateTime.MinValue, string.Empty, 0);

        public bool IsError() => Level >= 3;
        public string Format() => $"[{Timestamp:u}] ({Level}) {Message}";
    }

// NOTE: the closing brace for namespace Edge is intentionally omitted
// to produce a malformed-but-parseable file that must not cause a panic.
@@CASE@@ Generics
// Generics.cs — generic types and methods with various constraint forms.
using System;
using System.Collections.Generic;

namespace Demo.Generics
{
    /// <summary>A generic result type carrying a value or an error.</summary>
    public readonly struct Result<T, TError>
        where TError : Exception
    {
        public readonly T? Value;
        public readonly TError? Error;
        public bool IsOk => Error is null;

        public Result(T value) { Value = value; Error = default; }
        public Result(TError error) { Value = default; Error = error; }

        public TOut Match<TOut>(Func<T, TOut> onOk, Func<TError, TOut> onError)
        {
            return IsOk ? onOk(Value!) : onError(Error!);
        }
    }

    /// <summary>Generic repository with CRUD operations.</summary>
    public interface IRepository<TEntity, TKey>
        where TEntity : class
        where TKey : IEquatable<TKey>
    {
        TEntity? FindById(TKey id);
        IEnumerable<TEntity> FindAll();
        void Save(TEntity entity);
        bool Delete(TKey id);
    }

    /// <summary>In-memory generic repository.</summary>
    public class InMemoryRepository<TEntity, TKey> : IRepository<TEntity, TKey>
        where TEntity : class
        where TKey : IEquatable<TKey>
    {
        public const int InitialCapacity = 16;

        private readonly Dictionary<TKey, TEntity> _store;
        private readonly Func<TEntity, TKey> _keySelector;

        public InMemoryRepository(Func<TEntity, TKey> keySelector)
        {
            _store = new Dictionary<TKey, TEntity>(InitialCapacity);
            _keySelector = keySelector;
        }

        public TEntity? FindById(TKey id) =>
            _store.TryGetValue(id, out var e) ? e : null;

        public IEnumerable<TEntity> FindAll() => _store.Values;

        public void Save(TEntity entity)
        {
            var key = _keySelector(entity);
            _store.Add(key, entity);
        }

        public bool Delete(TKey id) => _store.Remove(id);
    }

    /// <summary>A generic pipeline builder.</summary>
    public sealed class Pipeline<TIn, TOut>
    {
        private readonly Func<TIn, TOut> _step;

        private Pipeline(Func<TIn, TOut> step) { _step = step; }

        public static Pipeline<TIn, TIn> Start() => new(x => x);

        public Pipeline<TIn, TNext> Then<TNext>(Func<TOut, TNext> next) =>
            new(input => next(_step(input)));

        public TOut Run(TIn input) => _step(input);
    }

    /// <summary>Extension methods with generic constraints.</summary>
    public static class GenericExtensions
    {
        public static TResult ApplyOrDefault<TSource, TResult>(
            this TSource? source,
            Func<TSource, TResult> selector,
            TResult defaultValue)
            where TSource : class
        {
            return source is null ? defaultValue : selector(source);
        }

        public static IEnumerable<T> WhereNotNull<T>(this IEnumerable<T?> source)
            where T : class
        {
            foreach (var item in source)
                if (item is not null)
                    yield return item;
        }

        public static void ForEach<T>(this IEnumerable<T> source, Action<T> action)
        {
            foreach (var item in source)
                action(item);
        }

        public static TAccum Fold<TItem, TAccum>(
            this IEnumerable<TItem> source,
            TAccum seed,
            Func<TAccum, TItem, TAccum> accumulator)
        {
            var result = seed;
            foreach (var item in source)
                result = accumulator(result, item);
            return result;
        }
    }

    /// <summary>Comparer factory for generic sorting.</summary>
    public static class ComparerFactory
    {
        public static IComparer<T> Create<T>(Func<T, T, int> compare) =>
            Comparer<T>.Create((a, b) => compare(a, b));

        public static IComparer<T> ByKey<T, TKey>(Func<T, TKey> keySelector)
            where TKey : IComparable<TKey>
        {
            return Comparer<T>.Create((a, b) =>
                keySelector(a).CompareTo(keySelector(b)));
        }
    }

    public enum SortDirection { Ascending, Descending }

    public delegate TResult Transformer<TIn, TOut, TResult>(TIn input, TOut context)
        where TResult : new();
}
@@CASE@@ Nested
// Deeply nested namespaces, classes, and inner types.
namespace Outer
{
    namespace Inner
    {
        namespace Deep
        {
            public class Root
            {
                public const int RootConst = 1;

                public int RootMethod(string s) { return s.Length; }

                public class ChildA
                {
                    private static readonly string Label = "child-a";

                    public string GetLabel() => Label;

                    public struct GrandchildStruct
                    {
                        public int X;
                        public int Y;

                        public GrandchildStruct(int x, int y)
                        {
                            X = x;
                            Y = y;
                        }

                        public override string ToString() => $"({X},{Y})";
                    }

                    public enum GrandchildEnum
                    {
                        Alpha,
                        Beta,
                        Gamma
                    }
                }

                public interface IChildB
                {
                    void DoWork();
                    int ComputeValue(double input);
                }

                protected class ChildC : IChildB
                {
                    public void DoWork() { }
                    public int ComputeValue(double input) => (int)Math.Round(input);

                    private sealed class GreatGrandchild
                    {
                        public const string Tag = "ggc";
                        public void Run() { }
                    }
                }
            }
        }
    }

    namespace Sibling
    {
        public static class SiblingHelper
        {
            public static int Add(int a, int b) => a + b;
            public static int Multiply(int a, int b) => a * b;
        }

        public abstract class SiblingBase
        {
            public abstract void Execute();
            public virtual string Describe() => "sibling-base";
        }

        public class ConcreteSibling : SiblingBase
        {
            public override void Execute() { }
            public override string Describe() => "concrete-sibling";
        }
    }
}
@@CASE@@ RealWorld
// A realistic ASP.NET-style web API controller and service layer.
using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Tasks;

namespace Acme.Commerce.Api
{
    /// <summary>
    /// Represents a product in the catalogue.
    /// </summary>
    public record Product(int Id, string Name, decimal Price, int Stock);

    /// <summary>
    /// Result returned by service operations.
    /// </summary>
    public record ServiceResult<T>(bool Success, T? Value, string? Error)
    {
        public static ServiceResult<T> Ok(T value) => new(true, value, null);
        public static ServiceResult<T> Fail(string error) => new(false, default, error);
    }

    /// <summary>
    /// Defines the contract for product persistence.
    /// </summary>
    public interface IProductRepository
    {
        Task<Product?> GetByIdAsync(int id, CancellationToken ct = default);
        Task<IReadOnlyList<Product>> ListAsync(int page, int pageSize, CancellationToken ct = default);
        Task<Product> AddAsync(Product product, CancellationToken ct = default);
        Task<bool> UpdateStockAsync(int id, int delta, CancellationToken ct = default);
        Task<bool> DeleteAsync(int id, CancellationToken ct = default);
    }

    /// <summary>
    /// Business logic layer wrapping the product repository.
    /// </summary>
    public class ProductService
    {
        public const int MaxPageSize = 100;
        public const int DefaultPageSize = 20;

        private static readonly TimeSpan CacheExpiry = TimeSpan.FromMinutes(5);

        private readonly IProductRepository _repo;

        public ProductService(IProductRepository repo)
        {
            _repo = repo ?? throw new ArgumentNullException(nameof(repo));
        }

        public async Task<ServiceResult<Product>> GetProductAsync(int id, CancellationToken ct = default)
        {
            var product = await _repo.GetByIdAsync(id, ct);
            return product is null
                ? ServiceResult<Product>.Fail($"Product {id} not found")
                : ServiceResult<Product>.Ok(product);
        }

        public async Task<ServiceResult<IReadOnlyList<Product>>> ListProductsAsync(
            int page = 1, int pageSize = DefaultPageSize, CancellationToken ct = default)
        {
            pageSize = Math.Clamp(pageSize, 1, MaxPageSize);
            var items = await _repo.ListAsync(page, pageSize, ct);
            return ServiceResult<IReadOnlyList<Product>>.Ok(items);
        }

        public async Task<ServiceResult<Product>> CreateProductAsync(
            string name, decimal price, int stock, CancellationToken ct = default)
        {
            if (string.IsNullOrWhiteSpace(name))
                return ServiceResult<Product>.Fail("Name is required");
            if (price < 0)
                return ServiceResult<Product>.Fail("Price must be non-negative");

            var product = new Product(0, name, price, stock);
            var created = await _repo.AddAsync(product, ct);
            return ServiceResult<Product>.Ok(created);
        }

        public async Task<ServiceResult<bool>> AdjustStockAsync(
            int id, int delta, CancellationToken ct = default)
        {
            var ok = await _repo.UpdateStockAsync(id, delta, ct);
            return ok
                ? ServiceResult<bool>.Ok(true)
                : ServiceResult<bool>.Fail($"Failed to adjust stock for product {id}");
        }
    }

    /// <summary>
    /// HTTP controller exposing the product service.
    /// </summary>
    [Route("api/products")]
    [ApiController]
    public class ProductsController
    {
        private readonly ProductService _service;

        public ProductsController(ProductService service)
        {
            _service = service;
        }

        [HttpGet]
        public async Task<IActionResult> GetAll(
            [FromQuery] int page = 1,
            [FromQuery] int pageSize = ProductService.DefaultPageSize,
            CancellationToken ct = default)
        {
            var result = await _service.ListProductsAsync(page, pageSize, ct);
            return result.Success ? Ok(result.Value) : StatusCode(500);
        }

        [HttpGet("{id:int}")]
        public async Task<IActionResult> GetById(int id, CancellationToken ct = default)
        {
            var result = await _service.GetProductAsync(id, ct);
            return result.Success ? Ok(result.Value) : NotFound(result.Error);
        }

        [HttpPost]
        public async Task<IActionResult> Create(
            [FromBody] CreateProductRequest req, CancellationToken ct = default)
        {
            var result = await _service.CreateProductAsync(req.Name, req.Price, req.Stock, ct);
            if (!result.Success)
                return BadRequest(result.Error);
            return CreatedAtAction(nameof(GetById), new { id = result.Value!.Id }, result.Value);
        }
    }

    /// <summary>Request body for product creation.</summary>
    public record CreateProductRequest(string Name, decimal Price, int Stock);

    /// <summary>
    /// Utility helpers.
    /// </summary>
    public static class PriceHelper
    {
        public static readonly decimal VatRate = 0.20m;
        public static readonly string CurrencySymbol = "£";

        public static decimal AddVat(decimal net) => net * (1 + VatRate);
        public static string Format(decimal amount) => $"{CurrencySymbol}{amount:F2}";
    }
}
@@CASE@@ Unicode
// Unicode.cs — non-ASCII identifiers in class, method, field and namespace names.
// C# allows any Unicode letter or digit in identifiers (plus @ prefix escapes).

using System;
using System.Collections.Generic;

namespace Büro.Verwaltung
{
    /// <summary>Grundlegende Konfiguration der Anwendung.</summary>
    public class Konfiguration
    {
        public const string Standardwert = "voreingestellt";
        public static readonly int MaximalAnzahl = 100;

        public string Name { get; set; } = string.Empty;
        public int Anzahl { get; private set; }

        public Konfiguration(string name, int anzahl)
        {
            Name = name;
            Anzahl = anzahl;
        }

        public bool IstGültig() => !string.IsNullOrEmpty(Name) && Anzahl > 0;

        public override string ToString() => $"{Name} (Anzahl={Anzahl})";
    }

    /// <summary>Verwaltet eine Sammlung von Elementen.</summary>
    public class Sammlung<T>
    {
        private readonly List<T> _elemente = new();

        public void Hinzufügen(T element) => _elemente.Add(element);

        public bool Enthält(T element) => _elemente.Contains(element);

        public int Anzahl => _elemente.Count;

        public T DiesesElement(int index) => _elemente[index];
    }

    /// <summary>Schnittstelle für verarbeitbare Einheiten.</summary>
    public interface IVerarbeitbar
    {
        void Verarbeiten();
        string Beschreiben();
        bool IstAbgeschlossen { get; }
    }

    public enum Zustand
    {
        Neu,
        InBearbeitung,
        Abgeschlossen,
        Fehlgeschlagen
    }

    public struct Koordinate
    {
        public double Breite;
        public double Länge;

        public Koordinate(double breite, double länge)
        {
            Breite = breite;
            Länge = länge;
        }

        public double Abstand(Koordinate andere)
        {
            double dx = Breite - andere.Breite;
            double dy = Länge - andere.Länge;
            return Math.Sqrt(dx * dx + dy * dy);
        }
    }
}

namespace 日本語.名前空間
{
    /// <summary>日本語の識別子を使ったクラス。</summary>
    public class 辞書クラス<キー, 値>
    {
        private readonly Dictionary<キー, 値> _データ = new();

        public void 追加(キー key, 値 value) => _データ[key] = value;

        public bool 含む(キー key) => _データ.ContainsKey(key);

        public 値 取得(キー key) => _データ[key];
    }

    public interface I検索可能<T>
    {
        T 検索(string クエリ);
        bool 存在する(string クエリ);
    }

    public static class テキストヘルパー
    {
        public const string 改行 = "\n";
        public static string 繰り返す(string text, int 回数) =>
            string.Concat(Enumerable.Repeat(text, 回数));
    }
}
@@CASE@@ expression_bodies
class Calc
{
    public int Add(int a, int b) => a + b;
    public double Half(double x) => x / 2;
    public int Block(int n) { return n; }
    public T Echo<T>(T value) => value;
}
@@CASE@@ sample
namespace App;

public class Account
{
    public const double Rate = 0.05;
    private double _balance;

    public Account(string id) { }

    public double Balance()
    {
        return _balance;
    }
}

public interface IGreeter
{
    string Greet();
}

public enum Status { Open, Closed }
@@CASE@@ operator_overloads
public struct S {
    public static S operator +(S a, S b) { return a; }
    public static bool operator ==(S a, S b) { return true; }
    public static bool operator !=(S a, S b) { return false; }
    public static S operator ++(S s) { return s; }
}
@@CASE@@ where_constraint
public class C {
    public void Before() {}
    public T Get<T>()
        where T : class
    {
        return default;
    }
    public void After() {}
}
@@CASE@@ interpolated_verbatim
class C
{
    public void Method()
    {
        string s = $@"
class FakeClass { }
{
";
    }
    public void Good() { }
}
