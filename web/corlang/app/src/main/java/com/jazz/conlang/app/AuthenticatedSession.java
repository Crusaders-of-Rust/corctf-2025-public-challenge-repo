package com.jazz.conlang.app;

import org.apache.wicket.authroles.authentication.AuthenticatedWebSession;
import org.apache.wicket.authroles.authorization.strategies.role.Roles;
import org.apache.wicket.injection.Injector;
import org.apache.wicket.request.Request;
import org.apache.wicket.spring.injection.annot.SpringBean;
import org.mindrot.jbcrypt.BCrypt;

import com.jazz.conlang.model.User;
import com.jazz.conlang.repo.UserRepository;

public class AuthenticatedSession extends AuthenticatedWebSession {

    @SpringBean
    private UserRepository userRepo;

    private String username;
    private Boolean isAdmin;

    public AuthenticatedSession(Request request) {
        super(request);
        Injector.get().inject(this);

    }

    @Override
    protected boolean authenticate(String username, String password) {
        User user = userRepo.findByUsername(username);
        if (user != null && BCrypt.checkpw(password, user.getPassword())) {
            this.username = username;
            this.isAdmin = user.getIsAdmin();
            return true;
        }
        return false;
    }

    @Override
    public Roles getRoles() {
        Roles roles = new Roles();
        if (isSignedIn()) {
            roles.add("USER");
            if (Boolean.TRUE.equals(isAdmin)) {
                roles.add("ADMIN");
            }
        }
        return roles;
    }

    public String getUsername() {
        return this.username;
    }

    public int getKarma() {
        User user = userRepo.findByUsername(this.username);
        if (user != null) {
            return user.getKarma();
        }
        return 0;
    }
}