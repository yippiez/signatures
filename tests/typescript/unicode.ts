/**
 * Non-ASCII identifiers and Unicode string content.
 * Covers: CJK, Arabic, Cyrillic, emoji-adjacent identifiers, accented Latin.
 */

export const 最大値: number = 100;
export const минимум: number = 0;
export const näherungswert: number = 3.14;
export const π: number = Math.PI;
export const τ: number = 2 * Math.PI;

export interface Форма {
  площадь(): number;
  периметр(): number;
}

export interface Şekil {
  alan(): number;
  çevre(): number;
}

export class Kreis implements Форма {
  constructor(public radius: number) {}

  площадь(): number {
    return π * this.radius * this.radius;
  }

  периметр(): number {
    return τ * this.radius;
  }
}

export class 円 {
  private 半径: number;

  constructor(半径: number) {
    this.半径 = 半径;
  }

  面積(): number {
    return π * this.半径 * this.半径;
  }

  周長(): number {
    return τ * this.半径;
  }
}

export class DaireŞekli implements Şekil {
  constructor(public yarıçap: number) {}

  alan(): number {
    return π * this.yarıçap * this.yarıçap;
  }

  çevre(): number {
    return τ * this.yarıçap;
  }
}

export type Зміна<T> = {
  старе: T;
  нове: T;
};

export function вітання(імʼя: string): string {
  return `Привіт, ${імʼя}!`;
}

export function grüßen(name: string): string {
  return `Hallo, ${name}!`;
}

export function résoudre(équation: string): number {
  return parseFloat(équation);
}

export function حساب(قيمة: number, معامل: number): number {
  return قيمة * معامل;
}

export const beschreibung: string = "Kreis mit Radius r";
export const описание: string = "Круг с радиусом r";
export const 説明: string = "半径rの円";
