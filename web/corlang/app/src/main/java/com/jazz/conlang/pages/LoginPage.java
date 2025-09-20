package com.jazz.conlang.pages;

import org.apache.wicket.markup.html.WebPage;
import org.apache.wicket.markup.html.form.Form;
import org.apache.wicket.markup.html.form.PasswordTextField;
import org.apache.wicket.markup.html.form.TextField;
import org.apache.wicket.markup.html.panel.FeedbackPanel;
import org.apache.wicket.model.Model;

import com.jazz.conlang.app.AuthenticatedSession;

public class LoginPage extends WebPage {

    public LoginPage() {
        Model<String> usernameModel = Model.of("");
        Model<String> passwordModel = Model.of("");

        Form<Void> form = new Form<>("loginForm") {
            @Override
            protected void onSubmit() {
                String username = usernameModel.getObject();
                String password = passwordModel.getObject();

                if (AuthenticatedSession.get().signIn(username, password)) {
                    setResponsePage(HomePage.class);
                } else {
                    error("Login failed. Invalid username or password.");
                }
            }
        };

        form.add(new TextField<>("username", usernameModel));
        form.add(new PasswordTextField("password", passwordModel));
        add(form);

        add(new FeedbackPanel("feedback"));
    }
}