#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define noreturn __attribute__((noreturn))

// les routines du défi
noreturn void rejeter(const char *message) {
    printf("%s\nRejected\n", message);
    exit(EXIT_FAILURE);
}

noreturn void accepter(const char *message) {
    printf("%s\nAccepted\n", message);
    exit(EXIT_SUCCESS);
}

// La file PEPS
#define TAILLE_DE_LA_FILE 4600
char file[TAILLE_DE_LA_FILE];
size_t tete;
size_t queue;

/**
 * Initialiser : initialise et vide la file.
 */
void initialiser() {
    tete = 0;
    queue = 0;
    memset(file, 0, sizeof(file));
}

/**
 * Enfiler : ajoute un élément dans la file.
 */
void enfiler(char c) {
    file[queue] = c;
    queue++;
    if (queue == TAILLE_DE_LA_FILE) queue = 0;

    if (queue == tete) {
        accepter("Interesting");
    }
}

/**
 * Défiler : retire le prochain élément de la file, et le renvoie.
 */
char defiler() {
    if (tete == queue) {
        rejeter("Dry");
    }

    char item = file[tete];
    tete++;
    if (tete == TAILLE_DE_LA_FILE) tete = 0;

    return item;
}

/**
 * La file est-elle vide ?
 * Renvoie vrai si la file est vide, faux sinon.
 */
int est_vide() {
    return tete == queue;
}

// Le système de tague (2-système)
#define PARAMETRE_DU_SYSTEME 2
typedef struct {
    char premiere_lettre;
    char *suffixe;
} regle_t;

#define NOMBRE_DE_REGLES 10
const regle_t REGLES[NOMBRE_DE_REGLES] = {
    {'a', "ns"},
    {'b', "jjj"},
    {'n', "a"},
    {'d', "q"},
    {'p', "cor"},
    {'f', "gg"},
    {'s', "aaa"},
    {'e', "tt"},
    {'j', "gg"},
    {'c', "flag"}
};

ssize_t chercher_regle(char premiere_lettre) {
    for (size_t i=0; i<NOMBRE_DE_REGLES; i++) {
        if (REGLES[i].premiere_lettre == premiere_lettre) {
            return i;
        }
    }
    return -1;
}

/**
 * Récrire : applique un transformation selon les règles
 * Renvoie 0 si on peut continuer, 1 si terminé
 */
int reecrire() {
    if (est_vide()) return 1;
    char premiere_lettre = defiler();
    ssize_t regle_choisi = chercher_regle(premiere_lettre);
    if (regle_choisi == -1) {
        rejeter("Bizzare");
    }

    for (int i=1; i<PARAMETRE_DU_SYSTEME; i++) {
        if (est_vide()) return 1;
        char c = defiler(); // supprimer la reste
        if (chercher_regle(c) == -1) {
            rejeter("Bizzarre");
        }
    }

    regle_t regle = REGLES[regle_choisi];

    // appliquer le règle
    for (size_t i=0; i<strlen(regle.suffixe); i++) {
        enfiler(regle.suffixe[i]);
    }
    
    return 0;
}

#ifndef ssize_t
typedef __ssize_t ssize_t;
#endif

int main() {
    printf("Enter flag:\n");
    char *saisie = NULL;
    size_t taille = 0;

    ssize_t taille_entree = getline(&saisie, &taille, stdin);
    if (taille_entree == -1) {
        rejeter("Illiterate");
    }

    // corctf{...}\n (au moins 9 chars)
    if (taille_entree < 9) {
        rejeter("Short");
    }

    if (taille_entree >= 40) {
        rejeter("Long");
    }

    if (strncmp("corctf{", saisie, 7)) {
        rejeter("Ineligible");
    }

    // terminaison: }\n
    if (strncmp("}\n", saisie+taille_entree-2, 2)) {
        rejeter("Ineligible");
    }

    initialiser();

    char *mot = saisie+7;
    size_t taille_restante = taille_entree - 9;
    for (size_t i=0; i<taille_restante; i++) {
        char lettre = mot[i];
        if ((i % 2) == 0) {
            if (lettre == 'c' || lettre > 'l') {
                rejeter("Forbidden");
            }
        } else if ((i % 6) < 4) {
            if (lettre < 'l' || lettre == 's') {
                rejeter("Forbidden");
            }
        } else {
            if (lettre < 'q') {
                rejeter("Forbidden");
            }
        }

        enfiler(lettre);
    }
    
    int fini;
    do {
        fini = reecrire();
    } while (!fini);
    
    // exited
    rejeter("Boring");
}