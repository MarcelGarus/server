topics = [ "this blog" ]

--start--

# Padded Code Sections on Websites

I'll just write a small post this month.
It's about code on websites, especially blog posts.

If I told you that lots of people read content on their phones while they're commuting, you're probably not surprised.
Well, that content includes programming articles.
Sadly, reading code on about half of the websites I encounter is sub-par and sometimes infuriating.

Even today, code is usually formatted with a column limit of 80 or 120.
The horizontal space on a phone is not enough for that.
So, horizontal scrolling it is.
But it gets worse:

![image]()

--snip--

Look at those paddings!
So much wasted space!
This is just sad.

I know it's impossible to spare your mobile readers the agony of constantly having to scroll back and forth for every single line.
But if you take one thing away from this post:
At least don't make it worse by taunting them with the space that could have be used more efficiently.

## How to do it in CSS

Let's assume you're using semantic elements and your HTML looks similar to this:

```html
<main>
    <article>
        <h1>Article</h1>
        <p>Some text</p>
        <code>Some code</code>
        <p>More text</p>
    </div>
</main>
```

Perhaps your CSS constrains the width of the `html:<article>` using a `css:max-width`:

```css
article {
    width: 100%;
    max-width: 50rem;
    margin: auto;
}
```

Now, instead of having a fixed amount of margin on the left and right of the article content (perhaps by adding some padding to the `html:<main>` element), you can make the children of the content take care of that spacing.
This way, we can add exceptions – such as code blocks.

```css
main {
    --padding: 1rem;
}

article>* {
    margin: 0 var(--padding);
}

article>code {
    margin: 0;
    padding: 1rem var(--padding);
}
```

Here, the `html:<code>` opts-out of the padding provided by the parent.
Instead, it adds its own padding, which will appear inside the code block and will scroll horizontally.
Beautiful!

![image]()
