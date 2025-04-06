// preamble ====================================================================

#import "@preview/diatypst:0.5.0": *
#show: slides.with(
  title: "A Guide to Optimization and Optimal Play in Greed",
  subtitle: "...via markov decision processes and dynamic programming",
  authors: ("Chance Addis"),
  count: "number"
)

// document ====================================================================

= Abstract

== Abstract

The game of Greed is a probabilistic two-player game. This paper aims to derive an optimal way to play the game - based on reasonable heuristic metrics - as well as as determine the optimal chance of success, as determined by out heuristic.

In other word, this paper aims to make you the best greed player, so continue on and lord your superior greed strategy over your enemies.

= How Do You Play Greed Again?

== Game Environment & Turns

Greed is a game that could be described as "kinda like blackjack with dice".

*Game parameters:*
- $M$: max score before going bust.
- $s$: number of sides on each die.

*Starts at* $s_p = 0$ and $s_0 = 0$ respectively.

Each turn, the player up to roll will choose some $n in NN^0$ number of dice to roll. Then the sum of those dice will be added to their score. Players will go back and forth like this until one of two terminal states is reached.

== How To Win

+ In the first option, a player’s score goes over $M$, i.e. and they go bust. In this case, they lose, so we’ll say that their rating---the heuristic measure that we use to measure how good a position is---is $0$, and thus their opponents rating is $1$.

+ In the second option, a player can choose to roll $0$ dice. This will initiate the last turn. The other player will then have one more chance to roll. The winner is the player who has the higher score that is not over $M$. In the case of a tie, rating is $1/2$ for each player.

= Mathematical Framework

== Mathematical Framework

Consider two types of states: terminal states, which denote the last turn of the game, and normal turns, representing all other states.

Normal and terminal states are conceptualized as $2$ by $2$ arrays with dimension $M + 1$ by $M + 1$, representing all possible states for player and opponent.

The x-axis designates the player presently up to roll, while the y-axis designates the player who has just rolled. Therefore, any state $S$ can be defined by the 3-tuple $(s_0, s_1, l)$

= PMF, PMF, PMF

== What Is The Random Variable

First, consider just a single dice. It has pmf $
  D_i^((s))(d) = cases(
    1 / s & "if" d in {1,dots, s} \
    0 & "otherwise"
  )
$ So let the random variable $T$ denote the sum of $n$ iid random dice, each with $s$ sides. It can thusly be written $
  T: (NN_0)^n -> RR, T := D_(1)^((s))(d_1) + dots.h.c + D_(n)^((s))(d_n)
$ Therefore, our goal is to find the pmf of $T$, which is notated $bold("p")_T^((n, s))(t)$, dependent on parameters $n, s$.

== Moment-Generating Functions

In order to find the probability mass function of $T$, we’ll use moment generating functions. Remember that probability distributions and moment generating functions have a one-to-one correspondence. So for a discrete random variable $X$ with probability distribution $
  f(x_i) = P(X = x_i) = p_i "for" i = 1, 2, dots, k
$ then it's mgf is $
  M_X (t) &= EE[e^(t X)] = sum_x e^(t x) dot.op f(x) \
  &= p_1 dot.op e^(t x_1) + dots.h.c + p_k dot.op e^(t x_k).
$ and visa versa (like in _HW 6, 2.1_).

== What is the MGF of $T$?

Recall the pmf of $D_(i)^((s))$. It's moment generating function is defined as $
  M_(D_i^((s)))(t) = EE[e^(t D_(i)^((s)))] = 1/s (e^t + e^(2 t) + dots.h.c + e^(s t))
$ Since $D_(i)^((s))$ are all independent and identically distributed, $
  M_(T)(t) &= EE[e^(t T)] =
  EE[e^(t (D_(1)^((s)) + dots.h.c + D_(n)^((s))))] =
  product_(i = 1)^n EE[ e^(t D_i^((s)))] \
  &= product_(i = 1)^n [1 / s (e^t + e^(2 t) + dots.h.c + e^(s t))] \
  &= 1 / s^n (e^t + e^(2 t) + dots.h.c + e^(s t))^n
$

== Coefficients of the Multinomials (Strap In)

We’ll write the multinomial in the nicer form $
  g(x) = 1/s_n (x + dots.h.c + x^s)^n
$ Our goal is to find the coefficient of the x t term.

Rewriting this with the geometric series  $
  g(x) = 1/s_n (x + dots.h.c + x^s)^n = 1/s_n ((x (x^s - 1)) / (x - 1))^n = 1/s_n (x^n (x^s - 1)^n) / (x - 1)^n
$

== Numerator

Looking at the numerator $(x^s - 1)^n$, we can expand this with the binomial theorem, _Fact D.2. [ASV]_ $
  (x^s + 1)^n = sum_(i = 0)^n binom(n, i) (x^s)^i dot.op (-1)^(n - i)
$ multiplying by $x^n$, this becomes $
  x^n (x^s + 1)^n = sum_(i = 0)^n binom(n, i) x^(s i + n) dot.op (-1)^(n - i)
$

== Denominator

Looking at the denominator $(x−1)^(−n)$. Rewrite this to $(−1)^(−n) dot (1−x)^(−n)$. Again this can be rewritten as the sum of binomial coefficients, but this time $(1−x)^(−n)$ is a binomial series (an expansion of the binomial theorem for complex exponents), in specific the negative binomial. Thus the denominator is $
  (x - 1)^n = (−1)^(−n) dot (1−x)^(−n) = (-1)^n sum_(j = 0)^(oo) binom(n+j-1, j) x^j
$

== Combine Variables

So together, the full equation of $g(x)$ becomes the product of the numerator and denominator as such $
  g(x) = (sum_(i = 0)^n binom(n, i) x^(s i + n) dot.op (-1)^(n - i)) ((-1)^n dot.op sum_(j = 0)^oo binom(n + j - 1, j) x^j)
$ Moving the $(- 1)^(- n)$ to the other equation and then simplifying $(- 1)^(- i)$ to $(- 1)^i$ $
  g(x) = (sum_(i = 0)^n (-1)^i binom(n, i) x^(s i + n)) (sum_(j = 0)^oo binom(n + j - 1, j) x^j)
$

== Cauchy Product

The finite sum can be though of as an infinite sum that takes $0$ whenever $i > n$. This allows us to use Cauchy product to get the coefficients, which generally states that: $
  (sum_(i = 0)^oo a_i x^i) (sum_(j = 0)^oo b_j x^j) = sum_(k = 0)^oo c_k x^k
$ Where the coefficients $c_k$ are defined as $
  c_k = sum_(l = 0)^k a_l b_(k - l)
$

== Combinatorics Magic

Doing some careful accounting of coefficients, this yields the double summation and result $
  g(x) = 1/s^n sum_(k = 0)^oo (sum_(l = 0)^(floor.l frac(k - n, s) floor.r) (-1)^k binom(n, k) binom(k - s l - 1, n - 1)) x^k
$ Which means that the coefficient of $x^t$ is $
  1/s^n sum_(l = 0)^(floor.l frac(t - n, s) floor.r) (-1)^k binom(n, k) binom(k - s l - 1, n - 1)
$

== Theorem

So finally, taking a step back, the pmf of T is the coefficient of the $i^"th"$ term, which is defined above. Thus the pmf of $T$ is defined $
  bold("p")_(T)^((n, s))(t) = 1/s^n sum_(k = 0)^(floor.l frac(t-n, s) floor.r) (-1)^k binom(n, k) binom(t-s dot.op k-1, n-1)
$

= Terminal States

== Defining a Rating Function on Terminal States

The goal is to find some $n$ to maximize the probability of getting a new score $s_p + t$ in the range $(s_o, M]$?

Or more precisely, given a state $(s_p , s_o , 1)$, what is the optimal $n$ such that the expected rating is maximized.

$
  text("rating")((s_p, s_o, 1), n) := sum_(t = s_o + 1)^(M) bold("p")_(T)^((n, s)) (t-s_p) + 1/2 dot.op bold("p")_T^((n, s)) (s_o - s_p)
$

where the summation describes the weighted sum of all next states given $n$, weighted according to their probability (transition matrix), hence rating is the expectation of its next possible states.

== Optimal Actions and Ratings on Terminal States

Since $n_star.op$ is the optimizer of rating, it is given by $
  n_(star)((s_p, s_o, 1)) := "argmax"_n text("rating")((s_p, s_o, 1), n)
$ Notice that the optimal rating comes for free. It’s the rating that was optimized for in finding $n_star.op$, so no additional work is required. $
  text("rating")_(star)((s_p, s_o, 1)) := text("rating")((s_p, s_o, 1), n_star)
$

= Calculating Terminal States

== Easy Cases

There are in fact certain states where the choice is immediately obvious without any need for calculations with pmfs. These states can be broken into 2 types that I’ll call _forfeit_ and _certain victory_.

In _forfeit states_, the opponent decided to end the game while they were behind, which guarantees that you win by doing nothing and rolling $0$ dice.

In _certain victory_ states, the relation between the player sp and oppoent so compared to the opponent and the maximum $M$ is such that for some $n$, all possible resulting sums $s_p + t$ are in the range $s_o < s_p + t <= M$. Thus there is some $n_star$ with a 100% win rate.

== Beginnings of the Algorithm

For most cases, a solution is not so simple. For these, the optimal $n_star$, $"rating"_star$ are found by the functions already formulated.

The algorithm for efficiently finding such maxima utilizes the properties of the distribution of $T$, to search minimal amount of n. The explanation for how and why is beyond the scope of this presentation, but you can read about it in the paper (though even then its a conjecture, turns out its hard to prove something when the distribution is constantly changing.)

= Optimizing Normal States

== Optimal Rating

The rating is defined in the same way as it is for terminal states: The expectation of the rating for the next possible states $S_1$ given some $n$.

*Example:*

Let’s give an example. Imagine that the optimal $n_star$ and $"rating"_star$ for every other state is known.

Consider rolling 2 dice at state (2, 6). You could end up at any of the following states: $S_1 = {(6, 4), (6, 5), (6, 6), (6, 7)}$. So since we know the rating of all these states, we can calculate the rating given $n$ by taking the weighted sum over $S_1$, with weights given by the pmf of $T$ , i.e. $bold("p")_T^((n,s))(2), ..., bold("p")_T^((n,s))(4)$.

== Defining a Rating Fucntion on Normal States

*Fact:* Rating is complementary with respect to each player’s rating for a given state.

Hence, the optimal rating for landing on a state $S$ is $1−"rating"(S, n_star)$ (since the rating of that state will be for the opponent), where $n_star$ is the optimal n for that state.

So the rating function is given by $
  text("rating")((s_p, s_o, 0), n) := cases(
    sum_(t = n)^(s dot.op n) 1 - text("rating")((s_o, s_p + t, 0), n_star.op) & "if" n > 0 \
    1 - text("rating") ((s_o, s_p, 1), n_star.op) & "if" n = 0)
$

== Optimal Actions and Ratings on Normal States

Thus the optimal $n_star$ given any possible state $S$, normal or terminal
is now defined to be $
  n_star = text("argmax")_n {
    sum_(t = n)^(s dot.op n) 1 - text("rating")(s_o, s_p + t, { 0 "if" n = 1 "else" 0 })
  }
$ with the rating function being either equation (1) or (4) depending on whether its a terminal state or normal state.

With $"rating"_star$ defined the same as for terminal states.

= Plots

== Background

#figure(image("assets/Screenshot 2023-12-09 at 21.07.12.png", width: 90.0%),
  caption: [
    Visual demonstration of terminal states
  ]
)

#figure(image("assets/Screenshot 2023-12-10 at 19.58.36.png", width: 80.0%),
  caption: [
    Distributions of $n = 2, 3, 4$ with $s = 6$
  ]
)

== Terminal states

#figure(image("assets/Screenshot 2023-12-12 at 20.21.12.png", width: 70.0%),
  caption: [
    Optimal actions and ratings for terminal states
  ]
)

== Normal States

#figure(image("assets/Screenshot 2023-12-14 at 01.43.15.png", width: 70.0%),
  caption: [
    Optimal actions and ratings for normal states
  ]
)

= References

== References

#bibliography("assets/references.yml", full: true)
