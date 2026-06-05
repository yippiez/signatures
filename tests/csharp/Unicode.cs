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
