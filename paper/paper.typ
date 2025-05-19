// preamble ====================================================================

#import "@preview/equals-template:0.2.0" as eq
#import eq.ctheorems: *
#show: eq.template(
  title: [A Guide to Optimization and Optimal Play in Greed],
  authors: ("Chance Addis",),
  abstract: [
    The game of Greed is a two-player, probabilistic zero-sum game. It involves rolling dice and accumulating scores, with the goal of reaching the closest score to $M$ without going over. This analysis examines Greed from the perspective of game theory to determine the optimal strategy at any state, using the framework of markov decision processes and dynamic programming.
  ]
)

// document ====================================================================

= Introduction to Greed <introduction-to-greed>

== What is Greed? <what-is-greed>

Greed is a game that could be described as "kinda like blackjack but with dice". Each player starts with a score of 0, which we'll label player score $s_p = 0$ and opponent score $s_o = 0$. The game also has some max score $M$ and, of course, each dice has $s$ sides. One player will start. They may choose any natural number $n in NN_0$ (including zero) of dice to roll. Once they roll, they will then add the sum of all the dice together and add that to their score. Then the opponent will choose to roll some $n$ number of dice and repeat, until one of two terminal states is reached.

In the first option, a player's score goes over $M$, i.e. and they go bust. In this case, they lose, so we'll say that their rating---the heuristic measure that we use to measure how good a position is---is $0$, and thus their opponents rating is $1$.

In the second option, a player can choose to roll $0$ dice. This will initiate the last turn. The other player will then have one more chance to roll. The winner is the player who has the higher score that is not over $M$. In the case of a tie, the rating is $1/2$ for each player.

So for now, rating is defined $
  "rating" = cases(
    1 & "if win" \
    1 / 2 & "if tie" \
    1 & "if lose"
  )
$ For clarity, all examples will use a game environment where $M = 10$ and $s = 3$, but our analysis will extend to any $s in NN_0, M in NN_0$. And notably, the classic game of greed has $M = 100, s = 6$.

== Why <why>

At this point, you may be asking why investigate optimal play in a completely unknown game that was most likely made up my Reed CS faculty?

I could explain that the optimization of probabilistic processes has relevance across various applications and industries, including finance, operations research, and engineering. Decision-making in uncertain environments is a common challenge, and optimizing strategies based on probabilistic models can lead to more informed and effective outcomes.

But come on. You didn't come to this paper to hear about the applications of game theory, and I didn't write this paper to talk about it. I wrote this paper because

+ I really like this problem. The idea of optimizing games is a exciting and challenging problem, and greed is just the kind of bit-sized game that allows for interesting problem-solving without becoming a behemoth (though I'd argue that it kinda is. The amount of rabbit holes that I've went down for this mini-thesis is insane.) and

+ My original algorithm lost to a basic if-else chain, and I'm filled with overwhelming spite. So much spite that I've spent the last 2 years thinking about it, only to spend #emph[this much] time optimizing it (given heuristic of rating / how states relate to each other - we'll get to it)

= Mathematical Framework <mathematical-framework>

Before beginning any calculations, it is necessary to first setup the problem space and how it will be conceptualized mathematically. By the nature of it's structure---its a markov process---Greed is well-split into two kinds of states; terminal states, which denote the last turn of the game, and normal turns, representing all other states.

Both normal and terminal states can be thought of as 2 by 2 boards with dimensions $M + 1$ by $M + 1$, representing all possible states for player and opponent. Notably, the x-axis designates the player presently up to roll, while the y-axis designates the player who has just rolled. So a state of $(5, 8)$ could mean you have $5$, and the opponent has $8$, or vise versa, depending on whose turn it is. This might seem like an unintuitive framework, but it well-suits this game because of how it alternates back and forth between players. Thus it avoids redundancy, because as we'll see later, rating is complementary.

Therefore, any state $S$ can be defined by the tuple $(s_0, s_1, l)$, where $s_0$ denotes the score of the player currently rolling, $s_1$ is the score of the other player, and $l$ is an indicator variable for whether the current turn is a terminal turn or normal turn.

For an example of how this all fits together, consider a scenario where the state is $(5, 8, 0)$, with your turn to roll. You roll $2$ dice, and get a sum of $4$. So the state is now $(8, 9, 0)$. Notice that the eight has gone from the second argument to the first. This is because it's the opponent's turn, so their score is listed as being "up to roll".

So in general, in some normal state $(s_0, s_1, 0)$, rolling $n > 0$ dice with sum $t$ makes the next state $(s_1, s_0 + t, 0)$, and in the case $n = 0$, the next state is $(s_1, s_0, 1)$. In some terminal state $(s_0, s_1, 1)$, rolling getting some sum $t$ will make the next state $(s_0, s_1 + t, 1)$, concluding the game.

= PMF <pmf>

Before beginning anything about actually deriving, it is imperative that we first find the most essential tool for solving this problem. What is the probability mass function of the sum of $n$ independent and identically distributed dice.

First, consider just a single dice. It has pmf $
  D_i^((s))(d) = cases(
    1 / s & "if" d in {1,dots, s} \
    0 & "otherwise"
  )
$ So let the random variable $T$ denote the sum of $n$ iid random dice, each with $s$ sides. It can thusly be written $
  T: (NN_0)^n -> RR, T := D_(1)^((s))(d_1) + dots.h.c + D_(n)^((s))(d_n)
$ Therefore, our goal is to find the pmf of $T$, which is notated $bold("p")_T^((n, s))(t)$, dependent on parameters $n, s$.

There are multiple ways to go about finding this pmf. One such method involves combinatorics. This paper will instead use moment generating functions and polynomials.

#theorem[
  Probability mass function of $T^((n, s)$ is defined $
  bold("p")_(T)^((n, s))(t) = 1/s^n sum_(k = 0)^(floor.l frac(t-n, s) floor.r) (-1)^k binom(n, k) binom(t-s dot.op k-1, n-1) $
]

But to get there, we're going to first need to prove some lemmas.

#lemma[Let $M_(T)^((n, s))$ be the moment generating function of $T$, then $
  M_(T)^((n, s)) = 1 / s^n (e^t + e^(2 t) + dots.h.c + e^(s t))^n $
]

#proof[
  Recall the pmf of $D_(i)^((s))$. It's moment generating function is defined as $
    M_(D_i^((s)))(t) = EE[e^(t D_(i)^((s)))] = 1/s (e^t + e^(2 t) + dots.h.c + e^(s t))
  $ Since $D_(i)^((s))$ are all independent and identically distributed, $
    M_(T)(t) &= EE[e^(t T)] =
    EE[e^(t (D_(1)^((s)) + dots.h.c + D_(n)^((s))))] =
    product_(i = 1)^n EE[ e^(t D_i^((s)))] \
    &= product_(i = 1)^n [1 / s (e^t + e^(2 t) + dots.h.c + e^(s t))] \
    &= 1 / s^n (e^t + e^(2 t) + dots.h.c + e^(s t))^n
  $

  Probability distributions and moment generating functions have a one-to-one correspondence. So for a discrete random variable $X$ with probability distribution $
    f(x_i) = P(X = x_i) = p_i "for" i = 1, 2,... k
  $ then it's mfg is $
    M_X (t) &= EE[e^(t X)] = sum_x e^(t x) dot.op f(x) \
    &= p_1 dot.op e^(t x_1) + dots.h.c + p_k dot.op e^(t x_k).
  $ and visa versa (like in _HW 6, 2.1_).

  It follows that for $t = 1$, the coefficients of the moment generating function are the probabilities of the random variable. Specifically, $P(X = x)$ is equal to the coefficient of the term $e^x$. Extending this principle to $T$, the coefficients of the multinomial outlines in *Lemma 1* correspond to the probabilities of rolling that corresponding sum, so $P(T = x)$ is still equal to the coefficient of the term $e^x$.

  For clarity, let's use the interchangable but more interpretable multinomial $1/s^n (x + x^2 + dots.h.c + x^s)^n$. The coefficients of $g(x)$ are the same as the coefficients of $M_T$, so the math is the same.
]

#lemma[For a multinomial $g(x) = 1/s^n (x + x^2 + dots.h.c + x^s)^n$, the coefficient of the $t^("th")$ term $x^t$ is given by $
  1/s^n sum_(k = 0)^(floor.l frac(t - n, s) floor.r) (-1)^k binom(n, k) binom(k - s k - 1, n - 1) $
]

#proof[
  Using the geometric series, rewrite $
    g(x) =
    1/s_n (x + dots.h.c + x^s)^n =
    1/s_n ((x (x^s - 1)) / (x - 1))^n =
    1/s_n (x^n (x^s - 1)^n) / (x - 1)^n
  $

  Looking at the numerator $(x^s - 1)^n$, we can expand this with the binomial theorem, _Fact D.2. [ASV]_ $
    (x^s + 1)^n = sum_(i = 0)^n binom(n, i) (x^s)^i dot.op (-1)^(n - i)
  $ multiplying by $x^n$, this becomes $
    x^n (x^s + 1)^n = sum_(i = 0)^n binom(n, i) x^(s i + n) dot.op (-1)^(n - i)
  $

  Looking at the denominator $(x - 1)^(- n)$. Rewrite this to $(- 1)^(- n) dot.op (1 - x)^(- n)$. Again this can be rewritten as the sum of binomial coefficients, but this time $(1 - x)^(- n)$ is a binomial series (an expansion of the binomial theorem for complex exponents), in specific the negative binomial. Starting by expanding the taylor series, $
    f(x) = (1 - x)^(-n) &= sum_(j = 0)^oo (f^(k)(0)) / j! x^j \
    &= 1 + n x + (n (n + 1)) / 2! x^2 + dots.h.c \
  $ and since $
    (n(n+1)(n+2) dots.h.c (n+j-1)) / j! = binom(n+j-1, j)
  $ that means $
    1 + n x + (n (n+1)) / 2! x^2 + dots.h.c = sum_(j = 0)^oo binom(n+j-1, j) x^j
  $ and thus the denominator is $
    (-1)^(-n) (x-1)^(-n) = (-1)^(-n) dot.op sum_(j = 0)^oo binom(n+j-1, j) x^j
  $

  So together, the full equation of $g(x)$ becomes the product of the numerator and denominator as such $
    g(x) = (sum_(i = 0)^n binom(n, i) x^(s i + n) dot.op (-1)^(n - i)) ((-1)^n dot.op sum_(j = 0)^oo binom(n + j - 1, j) x^j)
  $ Moving the $(- 1)^(- n)$ to the other equation and then simplifying $(- 1)^(- i)$ to $(- 1)^i$ $
    g(x) = (sum_(i = 0)^n (-1)^i binom(n, i) x^(s i + n)) (sum_(j = 0)^oo binom(n + j - 1, j) x^j)
  $

  The finite sum can be though of as an infinite sum that takes $0$ whenever $i > n$. This allows us to use Cauchy product to get the coefficients, which generally states that: $
    (sum_(i = 0)^oo a_i x^i) (sum_(j = 0)^oo b_j x^j) = sum_(k = 0)^oo c_k x^k
  $ Where the coefficients $c_k$ are defined as $
    c_k = sum_(l = 0)^k a_l b_(k - l)
  $

  Doing some careful accounting of coefficients (_authors note:_ I genuinely don't know what happened for this step), this yields the double summation and result $
    g(x) = 1/s^n sum_(k = 0)^oo (sum_(l = 0)^(floor.l frac(k - n, s) floor.r) (-1)^k binom(n, k) binom(k - s l - 1, n - 1)) x^k
  $ Which means that the coefficient of $x^t$ is $
    1/s^n sum_(l = 0)^(floor.l frac(t - n, s) floor.r) (-1)^k binom(n, k) binom(k - s l - 1, n - 1)
  $

  By recognizing that the coefficients of the multinomial directly represent the probabilities in the probability mass function (pmf), as previously explained, we can straightforwardly apply *lemma 2* to determine the pmf of $T$ is given by.
]

= Optimization of Terminal States <optimization-of-terminal-states>
#figure(image("assets/Screenshot 2023-12-09 at 21.07.12.png", width: 90.0%),
  caption: [
    Visual demonstration of terminal states
  ]
)

Considering that normal states will eventually terminate at terminal states, it is natural to first consider and calculate these states.

In order to find the optimal action in terminal states, it is useful to reframe the problem as “What is the optimal $n$ to maximize the probability of getting a new score $s_p + t$ between $s_o$, and $M$?\" Or more precisely, given a state $(s_p, s_o, 1)$, what is the optimal $n$ such that the expected rating is maximized.

Rating in this context is a metric that we use to determine the favorability of a state. We already defined the rating of winnign and losing, but here we extend rating further, to judge whether a state is more likely to result in winning or tying over losing. We define is as follows $
  text("rating")((s_p, s_o, 1), n) := sum_(t = s_o + 1)^(M) bold("p")_(T)^((n, s)) (t-s_p) + 1/2 dot.op bold("p")_T^((n, s)) (s_o - s_p)
$ where the summation describes the weighted sum of all next states given $n$, weighted according to their probability (transition matrix), hence rating is the expectation of its next possible states.

Since $n_star$ is the optimizer of rating, it is given by $
  n_star (s_0, s_1, 1) colon.eq upright("argmax")_n {upright("rating") ((s_0, s_1, 1), n)}
$

Notice that the optimal rating comes for free. It's the rating that was
optimized for in finding $n_star$, so no additional work is required. $
  text("rating")_star ((s_p, s_o, 1), n_star) := text("rating") ((s_p, s_o, 1), n_star)
$

= Calculating Terminal States <calculating-terminal-states>

Up to now, we've found a theoretical method for finding the optimal states. Unfortunately, results have to be concrete, so for practical computational purposes, it is crucial to determine a systematic and efficient approach. This would avoid the need to evaluate every possible $n$ from $s_o - s_p$ up to $M - s_p$, which is wildly impractical at larger board sizes. When should the algorithm stop?:w

There are in fact certain states where the choice is immediately obvious without any need for calculations with pmfs. These states can be broken into 2 types that I'll call _forfeit_ and _certain victory_.

In _forfeit_ states, the opponent decided to end the game while they were behind, which guarantees that you win by doing nothing and rolling 0 dice. This state will never occur in optimal Greed because a player that is behind would never roll 0 dice, since their rating would be 0, and they can always improve by rolling some optimum $n$ dice.

In _certain victory_ states, the relation between the player $s_p$ and opponent $s_o$ compared to the opponent and the maximum $M$ is such that for some $n$, all possible resulting sums $s_p + t$ are in the range $s_o < s_p + t <= M$. Thus there is some $n_star$ with a 100% win rate.

However, these states are edge cases in optimal greed. For the most part, all terminal states that are reached will not have a move which assured victory. These are the states where an optimization algorithm is needed.

In order to find that algorithm, first let's gain some insight into some of the properties of this distribution.

#proposition[$ EE[T^((s, n))] = n(s + 1) / 2 $]

#proof[$
  EE[T^((s, n))] &= EE[D_1^((s)) + dots.h.c + D_n^((s))] = EE[D_1^((s))] + dots.h.c + EE[D_n^((s))] \
  &= n dot.op EE[D_1^((s))] = n dot.op sum_(k = 1)^s k dot.op 1/s = n dot.op 1/s dot.op (s(s + 1)) / 2 \
  &= n(s + 1) / 2 $
]

#proposition[$ "Var"[T^((n, s))] = (s^2 - 1) / 12 $]

#proof[$
    text("Var")[T^((n, s))] &= text("Var")[D_1^((s)) + dots.h.c + D_n^((s))] \
    &= text("Var")[D_1^((s))] + dots.h.c + text("Var")[D_n^((s))] \
    &= n dot.op text("Var")[D_1^((s))]
  $

  The variance of $D_1^((s))$ is given by $EE[(D_1^((s)))^2] - EE[D_1^((s))]^2$ as stated in _Fact 3.48 [ASV]_. $
    EE[(D_1^((s)))^2] &= sum_(k = 1)^s k^2 dot.op 1 / s = 1/s (s(s + 1)(2 s + 1)) / 6 = ((s + 1) (2 s + 1)) / 6 \
    EE[ T^((n, s))] &= sum_(k = 1)^s k dot.op 1 / s = 1 / s dot.op frac((s) (s + 1), 2) = (s + 1) / 2
  $

  Thus the expectation of a single die $D_1^((s)$ is given by $
    text("Var")[D_1^((s))] &= EE[(D_1^((s))^2] - EE[D_1^((s))]^2 \
    &= ((s + 1)(2 s + 1)) / 6 - ((s + 1) / 2)^2 \
    &= (2 s^2 + 3 s + 1) / 6 - (s^2 + 2 s + 1) / 4 \
    &= (4 s^2 + 6 s + 2) / 12 - (3 s^2 + 6 s + 3) / 12 \
    &= (s^2 - 1) / 12
  $ Thus the variance of $T$ is $
    text("Var")[T^((s, n))] = n dot.op text("Var")[D_1^((s))] = n ((s^2 - 1) / 12)
  $
]

#figure(image("assets/Screenshot 2023-12-10 at 19.58.36.png", width: 80.0%),
  caption: [
    Distributions of $n = 2, 3, 4$ with $s = 6$
  ]
)

To derive an algorithm, this paper will make a few significant conjectures:

#conjecture[
  For any $n$ with corresponding $T^((n, s))$ with mean $s_p + EE[T^((n, s))]$ above $(s_o + M) / 2$, rating is monotonically decreasing as a function of $n$.
]

#conjecture[Rating as a function of $T^((n, s))$ with parameter $n$ is unimodal]

(_authors note:_ I settled on these higher level conjectures for brevity)

With this, deriving an algorithm is simple.

+ Find the smallest $n$ such that $s_p + EE[ T^((n, s))] > frac(s_o + M, 2)$ $
  & s_p + EE[T^((n, s))] > (s_o + M) / 2 => s_p + (n(s + 1)) / 2 > frac(s_o + M, 2) &  & "from lemma 3" \
  => & n(s + 1) / 2 > (s_o + M - 2 s_p) / 2 => n > (M + s_o - 2 s_p) / (s + 1) $

+ Calculate rating, keeping track of the highest one. When the rating starts to decrease, stop, and select the greatest $text("rating")_star$ and corresponding $n_star$

This will work because the rating distribution is unimodal, i.e. *conjecture 2*. From *conjecture 1*, starting above $n_0 = frac(M + s_o - 2 s_p, s + 1)$ means all $n > n_0$ have decreasing rating. This implies that $n_0$ must be to the left of the mean of the rating distribution. That means that when rating begins to decrease, we've transitioned from above the mean to below the mean. And since rating is unimodal, this must be the true maxima.

Applying the algorithm to the game with $M = 20, s = 3$, the terminal states are visualized below, with the label corresponding to the $n_star$ and the color corresponding to the rating.

#figure(image("assets/Screenshot 2023-12-12 at 20.21.12.png", width: 70.0%),
  caption: [
    Optimal actions and ratings for terminal states
  ]
)

= Optimization of Normal States <optimization-of-normal-states>

Finally, we can shift focus to normal states. Normal states are differentiable from terminal states because the player has to consider which states they could move to next, and how favorable those states are, and so on in a recursive manner, instead of just once.

And yet the way that rating is calculated is almost identical to terminal states: The expectation of the rating for the next possible states $S_1$ given some $n$. Which is, again, the weighted sum of the rating over all possible next states $S_1$, weighted by the transition matrix between the current and next states for $n$.

Let's give an example. Imagine that the optimal $n_star$ and
$text("rating")_star$ for every other state is known.

Consider rolling $2$ dice at state $(2, 6)$. You could end up at any of the following states: $S_1 = {(6, 4), (6, 5), (6, 6), (6, 7)}$. So since we know the rating of all these states, we can calculate the rating given $n$ by taking the weighted sum over $S_1$, with weights given by the pmf of $T$, i.e. $bold("p")_T^((n, s))(2), ..., bold("p")_T^((n, s))(4)$.

It's important to note that the rating at a resulting state like $(6, 4)$ represents the opponent's rating. However, rating is complementary. You have rating $P ("winning") + 1 / 2 P ("tying")$, and the opponent has rating $P ("losing") + 1 / 2 P ("tying")$. Thus $"player rating" + "opponent rating" = 1$, thus they are complementary.

Hence, the rating for landing on a state $S$ is $1 - text("rating")(S, n_star)$, where $n_star$ is the optimal $n$ for that state.

So the rating function is given by $
  text("rating")((s_p, s_o, 0), n) := cases(
    sum_(t = n)^(s dot.op n) 1 - text("rating")((s_o, s_p + t, 0), n_star) & "if" n > 0 \
    1 - text("rating") ((s_o, s_p, 1), n_star) & "if" n = 0)
$ Thus the optimal $n_star$ given any possible state $S$, normal or terminal is now defined to be $
  n_star = text("argmax")_n {
    sum_(t = n)^(s dot.op n) 1 - text("rating")(s_o, s_p + t, { 0 "if" n = 1 "else" 0 })
  }
$
with the rating function being either equation (1) or (4) depending on whether its a terminal state or normal state.

With $"rating"_star$ defined the same as equation (3).

_Remark:_ rating is guaranteed to produce a $n_star$ and $upright("rating")_star$ since eventually the state will either be terminal (which has concrete $n_star$ and $"rating"_star$) or go bust.

= Calculating Normal States <calculating-normal-states>
While the recursive approach with memoization, (the classic dynamic programming solution) would suffice as an algorithm to determine normal states, this either does a massive amount of duplicate work, or adds significant complexity. Instead, this paper suggests an iterative approach that utilizes a optimal ordering of calculating states.

Consider that we've already calculated optimal actions and ratings for terminal states. This means that the only unknowns are other normal states.

So what if we looked at the state $(M, M, 0)$? In this state, the only possible move would be either rolling $0$ and going to a terminal state, or going bust. Looking at the states adjacent to $(M, M, 0)$, these states can also go to a terminal state or go bust. They could also transition to $(M, M, 0)$, but that state is already calculated, so it still works.

In fact, because scores only increases, as the game goes on the sum of $s_p + s_o$ also increases. So if we calculate the states from the maximum possible sum $s_p + s_o$ and decrease, then we should never reach a state that we don't already know the answer to. This is the optimal ordering.

Along with the ordering, its also important to find a cutoff for checking $n$ values, like in terminal states. Since it's hard to know how normal states will effect each other, the upper bound will have to be less tight than it is for terminal states.

We can make this upper bound $n_0$ be where the mean $s_p + EE[T^((n, s))]$ is greater than $M$, calculated to be $
  s_p + EE[T^((n, s))] > M => s_p + frac(n (s + 1), 2) > M => n > (2 (M - s_p)) / (s + 1)
$ This works since any $n > n_0$ will have the exact same next states but
every probability will be smaller than it would be for $n_0$.

Applying the algorithm to the game with $M = 20, s = 3$, the normal states are visualized below, with the label corresponding to the $n_star$ and the color corresponding to the rating.

#figure(image("assets/Screenshot 2023-12-14 at 01.43.15.png", width: 70.0%),
  caption: [
    Optimal actions and ratings for normal states
  ]
)

= Conclusion <conclusion>
With both normal and terminal states calculated, we have thusly achieved our stated aim of finding some optimal strategy for playing greed. In terms of markov chains, we've tuned the parameter $n$ in order to optimize a transition matrix over the game states which optimizes for a heuristic that we defined called rating.

In doing so, we've defined a few significant functions. Primarily, we've defined a function for determining the rating of any possible state $S$, and from that also defining an optimal $n$ and rating, denoted $n_star$ and $upright("rating")_star$. In order to do so, we also derived the pmf for the sum of $n$ fair, independent and identically distributed dice.

As stated previously, there are a few limitations to this analysis. the biggest, of course, is that rating is not an objective function of success. A player could determine that tying is just as bad as winning, and have a unique, yet completely valid rating metric. Additionally, it is unclear whether defining rating as the weighted sum of the resulting states $S_1$ is truly the optimal way to relate ratings to each other.

As for additional research, there are many possible routes through which further research could proceed. Of primary relevance is a more mathematically rigorous framework. This paper briefly touches on many important and relevant ideas: markov decision chains, mathematical optimization for dynamic programming, reinforcement learning, etc. This problem has significant overlap with all of these fields. A more thorough analysis of this problem which incorporates these concepts would be a welcome addition.

Additionally, there are many open questions that further research could study. Top among them is to determine whether the conjectures described in "Calculating Terminal State" are in fact true, and thus whether the algorithm is sound. Another potential path is to investigate normal states, and see if there is some pattern which can help improve the upper bound.

As Greed is completely unexplored, there is massive room for further discovery and investigation in optimization, patterns, and more.

One additional note: A corollary of finding the optimal rating for every state is that we can determine whether going first has a benefit for rating, and how much. It turns out that for a game environment of $M = 10, s = 3$, going first has a rating of $0.54$, so it is very slightly advantageous to go first. And the rating implies that going first improves the chance of winning vs losing by about 4%. Take that conjecture as you will.

All code used to calculate the results is available here: #link("https://github.com/Approximately-Equal/Greed.git")

#bibliography("assets/references.yml", full: true)
