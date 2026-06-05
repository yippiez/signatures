package com.example.comments;

// class FakeInComment { void fakeMethod() {} }

/**
 * Javadoc with fake code:
 *   interface Hidden { void nope(); }
 *   static final int IGNORE = 99;
 */
public class CommentsStrings {

    public static final String GREETING = "Hello, World!";

    // private int notAField = 42;

    private String template;

    public CommentsStrings() {
        // class Inner {} -- this is in a comment, ignore it
        this.template = "class Fake { static final int X = 1; }";
    }

    public String buildQuery(String table) {
        /* interface InBlock { void m(); } */
        String query = "SELECT * FROM " + table + " WHERE class = 'active'";
        return query;
    }

    public String getTextBlock() {
        String block = """
                class InsideTextBlock {
                    static final int FAKE = 100;
                    void fakeMethod() {}
                }
                """;
        return block;
    }

    public int compute(int x) {
        String s = "enum Direction { NORTH, SOUTH }";
        /* enum Hidden { A, B } */
        // record FakeRecord(int a) {}
        return x * 2;
    }

    /*
     * block comment with:
     * public class MultiLineHidden {
     *     public void hiddenMethod() {}
     * }
     */
    public void realMethod() {
        String fake = "public record Ghost(int id) {}";
        System.out.println(fake);
    }
}

/* interface AlsoHidden { void x(); } */
