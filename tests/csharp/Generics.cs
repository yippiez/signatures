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
