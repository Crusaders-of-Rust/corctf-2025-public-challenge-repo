package com.jazz.conlang.util;

import java.util.Random;

public class CorLangGenerator {
    private static final String[] SYLLABLES = {
            "gla", "rya", "zor", "chop", "ba", "strel", "zork", "fizz", "ti", "wam", "ra", "mon", "day", "pron", "quin",
            "vel", "lemon", "gink", "rust", "nar", "jaz", "bex", "msfrog", "club", "pott", "uproj", "goomba", "d3v",
            "jam", "sice", "cor", "lemon", "qo", "0x", "peps"
    };
    private static final Random R = new Random();

    public static String generate() {
        return SYLLABLES[R.nextInt(SYLLABLES.length)]
                + SYLLABLES[R.nextInt(SYLLABLES.length)];
    }
}
