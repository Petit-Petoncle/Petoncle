# Tests Manuels pour Petoncle

## Tests Automatiques

Lance le script de test automatique:
```bash
~/.cargo/bin/cargo run
# Une fois dans le shell Petoncle:
chmod +x test_terminal.sh
./test_terminal.sh
```

## Tests Interactifs

### 1. ⌨️ Tests de Frappe de Base

**Test Tab Completion:**
```bash
cd s[TAB]          # Devrait compléter vers "src/"
ls Car[TAB]        # Devrait compléter vers "Cargo.toml"
ec[TAB]            # Devrait compléter vers "echo"
```

**Test Navigation avec Flèches:**
```bash
# Tape: echo "test 1"
# Appuie sur ↑ pour rappeler la commande
# Modifie avec ← et → pour naviguer
# Change "1" en "2" et execute
```

**Test Historique:**
```bash
pwd
ls
date
# Appuie plusieurs fois sur ↑ pour naviguer dans l'historique
# Appuie sur ↓ pour revenir
```

### 2. 🎮 Tests de Contrôles

**Test Ctrl+C (Interruption):**
```bash
sleep 100
# Appuie sur Ctrl+C (devrait interrompre)
```

**Test Ctrl+D (EOF):**
```bash
cat
# Tape du texte
# Appuie sur Ctrl+D (devrait terminer cat)
```

**Test Ctrl+L (Clear Screen):**
```bash
# Appuie sur Ctrl+L (devrait nettoyer l'écran)
```

**Test Ctrl+A et Ctrl+E:**
```bash
echo "test"
# Avant d'appuyer sur Enter:
# Ctrl+A devrait aller au début de la ligne
# Ctrl+E devrait aller à la fin
```

**Test Ctrl+U et Ctrl+K:**
```bash
echo "test text here"
# Avant d'appuyer sur Enter:
# Ctrl+U devrait effacer du curseur au début
# Ctrl+K devrait effacer du curseur à la fin
```

**Test Ctrl+W:**
```bash
echo one two three
# Avant Enter: Ctrl+W devrait effacer le mot précédent
```

### 3. 🔧 Tests de Pipes et Redirections

**Simple Pipe:**
```bash
echo "Hello World" | grep World
ls -la | head -5
ps aux | grep zsh
```

**Multiple Pipes:**
```bash
cat Cargo.toml | grep name | cut -d'"' -f2
ls -la | grep "^d" | wc -l
```

**Redirections:**
```bash
echo "test" > /tmp/test.txt
cat /tmp/test.txt
echo "append" >> /tmp/test.txt
cat /tmp/test.txt
cat < /tmp/test.txt
rm /tmp/test.txt
```

**Stderr Redirection:**
```bash
ls /nonexistent 2>/dev/null
ls /nonexistent 2>&1 | grep "No such"
```

### 4. 🎯 Tests de Commandes Composées

**Logical Operators:**
```bash
true && echo "Success"
false || echo "Fallback"
echo "One" ; echo "Two"
```

**Subshells:**
```bash
(cd /tmp && pwd)
pwd  # Devrait être toujours dans le répertoire original
```

**Command Substitution:**
```bash
echo "Today is $(date)"
files=$(ls | wc -l) && echo "Files: $files"
```

### 5. 🎨 Tests d'Affichage

**Couleurs:**
```bash
echo -e "\033[31mRouge\033[0m"
echo -e "\033[32mVert\033[0m"
echo -e "\033[33mJaune\033[0m"
```

**Emojis:**
```bash
echo "🚀 🐚 💻 ✨"
```

**Long Output:**
```bash
seq 1 100
ls -la /usr/bin
```

**Wide Output:**
```bash
echo "This is a very very very very very very very very very very very long line that should wrap properly"
```

### 6. 🔄 Tests de Boucles et Scripts

**For Loop:**
```bash
for i in {1..5}; do echo "Iteration $i"; done
```

**While Loop:**
```bash
i=0; while [ $i -lt 3 ]; do echo $i; i=$((i+1)); done
```

**If Statement:**
```bash
if [ -f "Cargo.toml" ]; then echo "Found"; else echo "Not found"; fi
```

### 7. 🔐 Tests de Caractères Spéciaux

**Quotes:**
```bash
echo 'single quotes'
echo "double quotes with $HOME"
echo "escaped \"quotes\""
```

**Special Characters:**
```bash
echo $HOME
echo $?
echo $$
echo $!
```

**Wildcards:**
```bash
ls *.toml
ls src/*.rs
echo test*
```

### 8. ⚡ Tests de Performance

**Commande Rapide:**
```bash
time echo "test"
```

**Output Volumique:**
```bash
time seq 1 10000
```

**Background Jobs:**
```bash
sleep 2 &
jobs
wait
```

### 9. 🪟 Tests de Redimensionnement

1. Lance Petoncle
2. Tape `tput cols; tput lines` pour voir la taille
3. Redimensionne la fenêtre du terminal
4. Tape à nouveau `tput cols; tput lines`
5. Vérifie que la taille est mise à jour

### 10. 🚪 Tests de Sortie

**Exit Normal:**
```bash
exit
# Devrait quitter proprement avec le message de sortie
```

**Ctrl+D:**
```bash
# Lance à nouveau et appuie sur Ctrl+D
# Devrait quitter proprement
```

**Exit avec Code:**
```bash
exit 42
# Devrait afficher le code de sortie 42
```

## 🎯 Checklist Finale

- [ ] Tab completion fonctionne
- [ ] Historique (↑↓) fonctionne
- [ ] Navigation (←→) fonctionne
- [ ] Ctrl+C interrompt les commandes
- [ ] Ctrl+D quitte le shell
- [ ] Pipes fonctionnent
- [ ] Redirections fonctionnent
- [ ] Couleurs s'affichent correctement
- [ ] Emojis s'affichent correctement
- [ ] Commandes longues wrappent correctement
- [ ] Background jobs fonctionnent
- [ ] Exit fonctionne proprement
- [ ] Pas de latence visible lors de la frappe
- [ ] Pas de caractères parasites à l'écran

## 🐛 Problèmes Connus à Surveiller

- [ ] Caractères dupliqués lors de la frappe rapide
- [ ] Curseur mal positionné après certain outputs
- [ ] Couleurs qui ne se réinitialisent pas
- [ ] Ctrl+C qui ne fonctionne pas sur certaines commandes
- [ ] Resize qui casse l'affichage
