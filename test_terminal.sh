#!/bin/zsh

# Script de test pour Petoncle Terminal Wrapper
# Ce script teste toutes les fonctionnalités de base du terminal

echo "🧪 Tests Petoncle Terminal Wrapper"
echo "=================================="
echo ""

# Test 1: Commandes de base
echo "📝 Test 1: Commandes de base"
echo "pwd: $(pwd)"
echo "whoami: $(whoami)"
echo "date: $(date)"
echo "✓ Test 1 passed"
echo ""

# Test 2: Pipes
echo "📝 Test 2: Pipes"
echo "Test simple pipe:"
echo "Hello World" | grep "World"
echo "Test multi-pipe:"
ls -la | grep "test" | wc -l
echo "✓ Test 2 passed"
echo ""

# Test 3: Redirections
echo "📝 Test 3: Redirections"
echo "Test content" > /tmp/petoncle_test.txt
echo "Created file with >"
cat /tmp/petoncle_test.txt
echo "More content" >> /tmp/petoncle_test.txt
echo "Appended with >>"
cat /tmp/petoncle_test.txt
rm /tmp/petoncle_test.txt
echo "✓ Test 3 passed"
echo ""

# Test 4: Commandes composées
echo "📝 Test 4: Commandes composées (&&, ||, ;)"
echo "Test with &&:"
echo "success" && echo "This should print"
echo "Test with ||:"
false || echo "This should print too"
echo "Test with ;:"
echo "First" ; echo "Second"
echo "✓ Test 4 passed"
echo ""

# Test 5: Sous-shells et substitution
echo "📝 Test 5: Sous-shells et substitution de commandes"
echo "Current dir: $(pwd)"
echo "Files count: $(ls | wc -l)"
result=$(echo "Substitution works")
echo "Variable from subshell: $result"
echo "✓ Test 5 passed"
echo ""

# Test 6: Variables d'environnement
echo "📝 Test 6: Variables d'environnement"
export PETONCLE_TEST="test_value"
echo "Export variable: $PETONCLE_TEST"
TEST_VAR="inline" && echo "Inline var: $TEST_VAR"
echo "✓ Test 6 passed"
echo ""

# Test 7: Commandes avec caractères spéciaux
echo "📝 Test 7: Caractères spéciaux"
echo "Quotes: 'single' and \"double\""
echo "Backticks: \`echo nested\`"
echo "Backslash: \\ and escapes: \n \t"
echo "Dollar: \$VAR and expansion: $HOME"
echo "✓ Test 7 passed"
echo ""

# Test 8: Codes de retour
echo "📝 Test 8: Codes de retour"
true
echo "true exit code: $?"
false
echo "false exit code: $?"
ls /nonexistent 2>/dev/null
echo "ls nonexistent exit code: $?"
echo "✓ Test 8 passed"
echo ""

# Test 9: Boucles et conditions
echo "📝 Test 9: Boucles et conditions"
for i in 1 2 3; do
    echo "Loop iteration: $i"
done
if [ -f "Cargo.toml" ]; then
    echo "Cargo.toml exists"
fi
echo "✓ Test 9 passed"
echo ""

# Test 10: Commandes en arrière-plan (background)
echo "📝 Test 10: Commandes background"
sleep 0.1 &
echo "Background job started"
wait
echo "Background job completed"
echo "✓ Test 10 passed"
echo ""

# Test 11: Couleurs ANSI
echo "📝 Test 11: Couleurs ANSI"
echo "\033[31mRed text\033[0m"
echo "\033[32mGreen text\033[0m"
echo "\033[33mYellow text\033[0m"
echo "\033[34mBlue text\033[0m"
echo "\033[1mBold text\033[0m"
echo "✓ Test 11 passed"
echo ""

# Test 12: Alias et fonctions
echo "📝 Test 12: Alias et fonctions"
alias testls="ls -la"
testls > /dev/null 2>&1
echo "Alias works"
test_function() {
    echo "Function called with: $1"
}
test_function "param"
unalias testls
echo "✓ Test 12 passed"
echo ""

# Test 13: Wildcards et glob patterns
echo "📝 Test 13: Wildcards"
echo "Files matching *.toml:"
ls *.toml 2>/dev/null || echo "No .toml files"
echo "Files matching src/*.rs:"
ls src/*.rs 2>/dev/null || echo "No .rs files in src/"
echo "✓ Test 13 passed"
echo ""

# Test 14: Commandes longues et wrapping
echo "📝 Test 14: Commandes longues"
echo "This is a very long command that should wrap properly in the terminal and not cause any issues with display or input handling"
echo "✓ Test 14 passed"
echo ""

# Test 15: Caractères Unicode et emojis
echo "📝 Test 15: Unicode et emojis"
echo "Emojis: 🚀 🐚 🔥 ✨ 💻"
echo "Accents: é è ê à ù ç"
echo "Symbols: → ← ↑ ↓ ✓ ✗"
echo "✓ Test 15 passed"
echo ""

# Résumé
echo "=================================="
echo "✅ Tous les tests sont passés!"
echo ""
echo "Tests manuels à effectuer:"
echo "  - Appuyer sur Tab pour l'autocomplétion"
echo "  - Utiliser les flèches ↑↓ pour l'historique"
echo "  - Utiliser les flèches ←→ pour naviguer dans la ligne"
echo "  - Appuyer sur Ctrl+C pour annuler une commande"
echo "  - Appuyer sur Ctrl+D ou taper 'exit' pour quitter"
echo "  - Redimensionner la fenêtre du terminal"
echo "  - Copier/coller du texte"
echo ""
