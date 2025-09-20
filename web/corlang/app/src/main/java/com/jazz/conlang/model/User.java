package com.jazz.conlang.model;

import java.io.Serializable;
import java.util.Optional;

import jakarta.persistence.Entity;
import jakarta.persistence.Id;
import jakarta.persistence.Table;

@Entity
@Table(name = "conlang_user")
public class User implements Serializable {
    @Id
    private String username;
    private String password;
    private Integer karma;
    private Boolean isAdmin;

    public User() {
    }

    public String getPassword() {
        return password;
    }

    public Boolean getIsAdmin() {
        return isAdmin;
    }

    public String getUsername() {
        return username;
    }

    public Integer getKarma() {
        return Optional.ofNullable(karma).orElse(0);
    }
    
    public Integer incrementKarma() {
        if (this.karma == 9) {
            this.promoteToAdmin();
        }
        return this.karma++;
    }

    public Integer decrementKarma() {
        if (this.karma != 0) {
            this.karma--;
        }
        return this.karma;
    }

    private Void promoteToAdmin() {
        this.isAdmin = true;
        return null;
    }
}