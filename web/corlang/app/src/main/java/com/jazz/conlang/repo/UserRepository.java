package com.jazz.conlang.repo;

import org.springframework.data.repository.CrudRepository;

import com.jazz.conlang.model.User;

public interface UserRepository extends CrudRepository<User, String> {
    User findByUsername(String username);
}
