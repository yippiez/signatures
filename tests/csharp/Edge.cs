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
