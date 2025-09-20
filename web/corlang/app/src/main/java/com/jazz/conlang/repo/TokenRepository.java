package com.jazz.conlang.repo;

import org.springframework.data.repository.CrudRepository;

import com.jazz.conlang.model.Token;

public interface TokenRepository extends CrudRepository<Token, Long> {
    Token findByTokenName(String tokenName);
}