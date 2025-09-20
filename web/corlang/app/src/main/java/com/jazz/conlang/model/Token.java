package com.jazz.conlang.model;

import jakarta.persistence.Entity;
import jakarta.persistence.Table;
import jakarta.persistence.Id;

@Entity
@Table(name = "conlang_token")
public class Token {
    @Id
    private Long id;
    private String tokenName;
    private String tokenValue;

    public String getValue() {
        return tokenValue;
    }
}