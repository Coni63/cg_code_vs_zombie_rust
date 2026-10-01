# Code vs Zombies - Game Documentation

## 🎯 Objectif

Détruisez les zombies rapidement pour gagner un maximum de points et assurez-vous de garder au moins un humain en vie.

---

## 📜 Règles de base

* **Terrain de jeu :** Grille de **16000 x 9000** unités (origine `0 0` dans le coin supérieur gauche).
* **Personnage principal (Ash) :**
  * Vous contrôlez Ash. À chaque tour, vous spécifiez une coordonnée cible `X Y`.
  * **Déplacement :** Ash se déplace de **1000 unités** par tour vers la cible (ou directement dessus s'il est à moins de 1000 unités).
  * **Tir :** À la fin de son déplacement, Ash détruit automatiquement tout zombie présent dans un **rayon de 2000 unités**.
* **Humains :**
  * Les humains sont immobiles.
  * Si tous les humains meurent, vous **perdez la partie** et obtenez un score de 0.
* **Zombies :**
  * **Déplacement :** À chaque tour, chaque zombie se déplace de **400 unités** vers l'humain le plus proche (Ash inclus).
  * **Attaque :** Si un zombie arrive à moins de 400 unités d'un humain, il se déplace sur ses coordonnées et le tue.
  * Plusieurs zombies peuvent occuper la même coordonnée.

---

## 🔄 Ordre d'exécution d'un tour

1. **Déplacement des zombies** vers leurs cibles respectives (400 unités).
2. **Déplacement de Ash** vers sa coordonnée cible (1000 unités).
3. **Attaque de Ash :** Destruction de tous les zombies à $\le 2000$ unités de Ash.
4. **Attaque des zombies :** Destruction des humains atteints par des zombies.

---

## 🏆 Conditions de fin de partie

* **Victoire :** Tous les zombies sont détruits et au moins un humain est encore en vie.
* **Défaite :** Tous les humains sont tués (Ash exclu).

---

## 🧮 Système de points

1. **Valeur de base d'un zombie :**
   

   $$
   \text{Valeur} = 10 \times (\text{Nombre d'humains vivants})^2
   $$

   *(Note : Ash ne compte pas parmi les humains).*

2. **Multiplicateur de Combo (Suite de Fibonacci) :**
   Si vous détruisez plusieurs zombies au cours d'un même tour, la valeur du $n^{\text{ème}}$ zombie éliminé est multipliée par le $(n+2)^{\text{ème}}$ terme de la suite de Fibonacci ($F_3 = 2$, $F_4 = 3$, $F_5 = 5$, $F_6 = 8$, etc.).
   
   *Astuce : Tuez un maximum de zombies dans le même tour pour exploser votre score !*

---

## 🧠 Rules / Precision pour experts

* **Arrondi des coordonnées :** Le jeu utilise uniquement des coordonnées entières. Si un déplacement donne un résultat décimal, la valeur est **arrondie à l'entier inférieur le plus proche** ($\lfloor x \rfloor, \lfloor y \rfloor$).
* **Prédiction :** L'entrée standard fournit directement les coordonnées futures (`zombieXNext`, `zombieYNext`) de chaque zombie pour le tour suivant.

---

## 📥 Entrées et Sorties du Jeu

### Boucle de jeu (Chaque tour)

#### Entrées :
* **Ligne 1 :** `x y` (Coordonnées actuelles de Ash).
* **Ligne 2 :** `humanCount` (Nombre d'humains encore en vie).
* **`humanCount` lignes suivantes :** `humanId humanX humanY`
* **Ligne suivante :** `zombieCount` (Nombre de zombies restants).
* **`zombieCount` lignes suivantes :** `zombieId zombieX zombieY zombieXNext zombieYNext`

#### Sorties :
* **1 ligne :** `targetX targetY [message]`
  * Deux entiers représentant la destination souhaitée pour Ash. Un message optionnel peut être ajouté à la fin.

---

## ⏱️ Contraintes

* $0 \le x < 16000$
* $0 \le y < 9000$
* $1 \le \text{humanCount} < 100$
* $1 \le \text{zombieCount} < 100$
* **Temps de réponse maximal :** $\le 100\text{ ms}$ par tour.


## Test code:

```
cargo run < testcases/test0X.txt
cargo run --release < testcases/test0X.txt
```