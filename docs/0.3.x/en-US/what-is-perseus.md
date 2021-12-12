# What is Perseus?

If you're familiar with [NextJS](https://nextjs.org), Perseus is that for Wasm. If you're familiar with [SvelteKit](https://kit.svelte.dev), it's that for [Sycamore](https://github.com/sycamore-rs/sycamore).

If none of that makes any sense, this is the section for you! If you're not in the mood for a lecture, there's a TL;DR at the bottom of this page!

### Rust web development

[Rust](https://www.rust-lang.org/) is an extremely powerful programming language, but I'll leave the introduction of it [to its developers](https://www.rust-lang.org/).

[WebAssembly](https://webassembly.org) (abbreviated Wasm) is like a low-level programming language for your browser. This is revolutionary, because it allows websites and web apps to be built in programming languages other than JavaScript. Also, it's [really fast](https://medium.com/@torch2424/webassembly-is-fast-a-real-world-benchmark-of-webassembly-vs-es6-d85a23f8e193) (usually >30% faster than JS).

But developing directly for the web with Rust using something like [`web-sys`](https://docs.rs/web-sys) isn't a great experience, it's generally agreed in the web development community that developer experience and productivity are vastly improved by having a _reactive_ framework. Let's approach this from a traditional JavaScript and HTML perspective first.

Imagine you want to create a simple counter. Here's how you might do it in a non-reactive framework (again, JS and HTML here, no Rust yet):

```html
<p id="counter">0</p>
<br />
<button
    onclick="document.getElementById('counter').innerHTML = parseInt(document.getElementById('counter').innerHTML) + 1"
>
    Increment
</button>
```

If you're unfamiliar with HTML and JS, don't worry. All this does is create a paragraph with a number inside and then increment it. But the problem is clear in terms of expression: why can't we just put a variable in the paragraph and have that re-render when we increment that variable? Well, that's reactivity!

In JS, there are frameworks like [Svelte](https://svelte.dev) and [ReactJS](https://reactjs.org) that solve this problem, but they're all bound significantly by the language itself. JavaScript is slow, dynamically typed, and [a bit of a mess](https://medium.com/netscape/javascript-is-kinda-shit-im-sorry-2e973e36fec4). Like all things to do with the web, changing things is really difficult because people have already started using them, and there will always be _someone_ still using Internet Explorer, which supports almost no modern web standards at all.

[Wasm](https://webassembly.org) solves all these problems by creating a unified format that other programming languages, like Rust, can compile into for the browser environment. This makes websites safer, faster, and development more productive. The equivalent of these reactive frameworks for Rust in particular would be projects like [Sycamore](https://sycamore-rs.netlify.app), [Seed](https://seed-rs.org), and [Yew](https://yew.rs). Sycamore is the most extensible and low-level of those options, and it's more performant because it doesn't use a [virtual DOM](https://svelte.dev/blog/virtual-dom-is-pure-overhead) (link about JS rather than Rust), and so it was chosen to be the backbone of Perseus. Here's what that counter might look like in [Sycamore](https://sycamore-rs.netlify.app) (the incrementation has been moved into a new closure for convenience):

```rust
use sycamore::prelude::*;

let counter = Signal::new(0);
let increment = cloned!((counter) => move |_| counter.set(*counter.get() + 1));

template! {
    p { (counter.get()) }
    button(on:click = increment) { "Increment" }
}
```

You can learn more about Sycamore's amazing systems [here](https://sycamore-rs.netlify.app).

### This sounds good...

But there's a catch to all this: rendering. With all these approaches in Rust so far (except for a few mentioned later), all your pages are rendered _in the user's browser_. That means your users have to download your Wasm code and run it before they see anything at all on their screens. Not only does that increase your loading time ([which can drive away users](https://medium.com/@vikigreen/impact-of-slow-page-load-time-on-website-performance-40d5c9ce568a)), it reduces your search engine rankings as well.

This can be solved through _server-side rendering_ (SSR), which means that we render pages on the server and send them to the client, which means your users see something very quickly, and then it becomes _interactive_ (usable) a moment later. This is better for user retention (shorter loading times) and SEO (search engine optimization).

The traditional approach to SSR is to wait for a request for a particular page (say `/about`), and then render it on the server and send that to the client. This is what [Seed](https://seed-rs.org) (an alternative to Perseus) does. However, this means that your website's _time to first byte_ (TTFB) is slower, because the user won't even get _anything_ from the server until it has finished rendering. In times of high load, that can drive loading times up worryingly.

The solution to this is _static site generation_ (SSG), whereby your pages are rendered _at build time_, and they can be served almost instantly on any request. This approach is fantastic, and thus far widely unimplemented in Rust. The downside to this is that you don't get as much flexibility, because you have to render everything at build time. That means you don't have access to any user credentials or anything else like that. Every page you render statically has to be the same for every user.

Perseus supports SSR _and_ SSG out of the box, along with the ability to use both on the same page, rebuild pages after a certain period of time (e.g. to update a list of blog posts every 24 hours) or based on certain conditions (e.g. if the hash of a file has changed), or even to statically build pages on demand (the first request is SSR, all the rest are SSG), meaning you can get the best of every world and faster build times.

To our knowledge, the only other framework in the world right now that supports this feature set is [NextJS](https://nextjs.org) (with growing competition from [GatsbyJS](https://www.gatsbyjs.com)), which only works with JavaScript. Perseus goes above and beyond this for Wasm by supporting whole new combinations of rendering options not previously available, allowing you to create optimized websites and web apps extremely efficiently.

## How fast is it?

[Benchmarks show](https://rawgit.com/krausest/js-framework-benchmark/master/webdriver-ts-results/table.html) that [Sycamore](https://sycamore-rs.netlify.app) is slightly faster than [Svelte](https://svelte.dev) in places, one of the fastest JS frameworks ever. Perseus uses it and [Actix Web](https://actix.rs) or [Warp](https://github.com/seanmonstar/warp) (either is supported), some of the fastest web servers in the world. Essentially, Perseus is built on the fastest tech and is itself made to be fast.

The speed of web frameworks is often measured by [Lighthouse](https://developers.google.com/web/tools/lighthouse) scores, which are scores out of 100 (higher is better) that measure a whole host of things, like *total blocking time*, *first contentful paint*, and *time to interactive*. These are then aggregated into a final score and grouped into three brackets: 0-49 (slow), 50-89 (medium), and 90-100 (fast). This website, which is built with Perseus, using [static exporting](:exporting) and [size optimizations](:deploying/size), consistently scores a 100 on desktop and above 90 for mobile. You can see this for yourself [here](https://developers.google.com/speed/pagespeed/insights/?url=https%3A%2F%2Farctic-hen7.github.io%2Fperseus%2Fen-US%2F&tab=desktop) on Google's PageSpeed Insights tool.

<details>
<summary>Why not 100 on mobile?</summary>

The only issue that prevents Perseus from achieving a consistent perfect score on mobile is *total blocking time*, which measures the time between when the first content appears on the page and wehn that content is interactive. Of course, WebAssembly code is used for this part (compiled from Rust), which isn't completely optimized for yet on many mobile devices. As mobile browsers get better at parsing WebAssembly, TBT will likely decrease further from the medium range to the green range (which we see for more poerful desktop systems).

</details>

If you're interested in seeing how Perseus compares on speed and a number of other features to other frameworks, check out the [comparisons page](comparisons).

## How convenient is it?

Perseus aims to be more convenient than any other Rust web framework by taking an approach similar to that of [ReactJS](https://reactjs.org). Perseus itself is an extremely complex system consisting of many moving parts that can all be brought together to create something amazing, but the vast majority of apps don't need all that customizability, so we built a command-line interface (CLI) that handles all that complexity for you, allowing you to focus entirely on your app's code.

Basically, here's your workflow:

1. Create a new project.
2. Define your app in around 12 lines of code and some listing.
3. Code your amazing app.
4. Run `perseus serve`.

## How stable is it?

Perseus is considered reasonably stable at this point, though it still can't be recommended for *production* usage just yet. That said, this very website is built entirely with Perseus and Sycamore, and it works pretty well!

For now though, Perseus is perfect for anything that doesn't face the wider internet, like internal tools, personal projects, or the like. Just don't use it to run a nuclear power plant, okay?

## Summary

If all that was way too long, here's a quick summary of what Perseus does and why it's useful!

-   JS is slow and a bit of a mess, [Wasm](https://webassembly.org) lets you run most programing languages, like Rust, in the browser, and is really fast
-   Doing web development without reactivity is really annoying, so [Sycamore](https://sycamore-rs.netlify.app) is great
-   Perseus lets you render your app on the server, making the client's experience _really_ fast, and adds a ton of features to make that possible, convenient, and productive (even for really complicated apps)
