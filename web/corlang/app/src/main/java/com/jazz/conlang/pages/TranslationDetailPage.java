package com.jazz.conlang.pages;

import org.apache.wicket.authroles.authorization.strategies.role.annotations.AuthorizeInstantiation;
import org.apache.wicket.markup.html.WebPage;
import org.apache.wicket.markup.html.basic.Label;
import org.apache.wicket.markup.html.form.Button;
import org.apache.wicket.markup.html.form.Form;
import org.apache.wicket.markup.html.form.PasswordTextField;
import org.apache.wicket.markup.html.panel.FeedbackPanel;
import org.apache.wicket.model.Model;
import org.apache.wicket.request.mapper.parameter.PageParameters;
import org.apache.wicket.spring.injection.annot.SpringBean;

import com.jazz.conlang.model.Translation;
import com.jazz.conlang.model.User;
import com.jazz.conlang.repo.TokenRepository;
import com.jazz.conlang.repo.TranslationRepository;
import com.jazz.conlang.repo.UserRepository;

@AuthorizeInstantiation("ADMIN")
public class TranslationDetailPage extends WebPage {

    @SpringBean
    private TranslationRepository repo;

    @SpringBean
    private UserRepository userRepo;

    @SpringBean
    private TokenRepository tokenRepo;

    private User author;

    private String formatAuthorInfo(User author) {
        String role = author.getIsAdmin() ? "Admin" : "User";
        return String.format("%s (%s) - %d karma",
                author.getUsername(),
                role,
                author.getKarma());
    }

    public TranslationDetailPage(PageParameters parameters) {
        Long translationId = parameters.get("id").toLong();
        Translation translation = repo.findById(translationId).orElse(null);

        if (translation == null) {
            setResponsePage(AdminPage.class);
            return;
        }

        author = userRepo.findByUsername(translation.getAuthor());

        add(new FeedbackPanel("feedback"));
        add(new Label("keyName", translation.getKeyName()));
        add(new Label("locale", translation.getLocaleTag()));
        add(new Label("proposed", translation.getValue()));
        add(new Label("rendered", translation.render(this)));
        add(new Label("approved", String.valueOf(translation.isApproved())));
        add(new Label("author", formatAuthorInfo(author)));

        Model<String> tokenModel = Model.of("");

        Form<Void> approvalForm = new Form<>("approvalForm");
        add(approvalForm);

        approvalForm.add(new PasswordTextField("approvalToken", tokenModel).setRequired(false));

        approvalForm.add(new Button("approve") {
            @Override
            public void onSubmit() {
                /* Check if admin provided the approval token */
                String token = tokenModel.getObject();
                String expectedToken = tokenRepo.findByTokenName("approval_token").getValue();
                if (token == null || !token.equals(expectedToken)) {
                    error("Approval token incorrect.");
                    return;
                }
                /* Set translation to approved */
                translation.setApproved(true);
                repo.save(translation);

                /* Increment the karma of the author */
                author.incrementKarma();
                userRepo.save(author);

                info("Translation approved!");
                setResponsePage(AdminPage.class);
            }
        });

        approvalForm.add(new Button("reject") {
            @Override
            public void onSubmit() {
                /* Delete the shitty translation */
                repo.delete(translation);

                /* Decrement the karma of the author */
                author.decrementKarma();
                userRepo.save(author);

                setResponsePage(AdminPage.class);
            }
        });
    }
}
