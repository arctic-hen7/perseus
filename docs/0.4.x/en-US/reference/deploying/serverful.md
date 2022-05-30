# Server Deployment

If your app uses rendering strategies that need a server, you won't be able to export your app to purely static files, and so you'll need to host the Perseus server itself.

You can prepare your production server by running `perseus deploy`, which will create a new directory called `pkg/`, which will contain the standalone binary and everything needed to run it.

## Hosting Providers

As you may recall from [this section](:reference/stores) on immutable and mutable stores, Perseus modifies some data at runtime, which is problematic if your hosting provider imposes the restriction that you can't write to the filesystem (as Netlify does). Perseus automatically handles this as well as it can by separating out mutable from immutable data, and storing as much as it can on the filesystem without causing problems. However, data for pages that use the _revalidation_ or _incremental generation_ strategies must be placed in a location where it can be changed while Perseus is running.

If you're only using _build state_ and/or _build paths_ (or neither), you should export your app to purely static files instead, which you can read more about doing [here](:reference/exporting). That will avoid this entire category of problems, and you can deploy basically wherever you want.

If you're bringing _request state_ into the mix, you can't export to static files, but you can run on a read-only filesystem, because only the _revalidation_ and _incremental generation_ strategies require mutability. Perseus will use a mutable store on the filesystem in the background, but won't ever need it.

If you're using _revalidation_ and _incremental generation_, you have two options, detailed below.

### Writable Filesystems

The first of these is to use an old-school provider that gives you a filesystem that you can write to. This may be more expensive for hosting, but it will allow you to take full advantage of all Perseus' features in a highly performant way.

You can deploy to one of these providers without any further changes to your code, as they mimic your local system almost entirely (with a writable filesystem). Just run `perseus deploy` and copy the resulting `pkg/` folder to the server!

### Alternative Mutable Stores

The other option you have is deploying to a modern provider that has a read-only filesystem and then using an alternative mutable store. That is, you store your mutable data in a database or the like rather than on the filesystem. This requires you to implement the `MutableStore` `trait` for your storage system (see the [API docs](https://docs.rs/perseus)), which should be relatively easy.

You can then provide this to `PerseusApp` with the `.new_with_mutable_store()` function, which must be run on `PerseusAppWithMutableStore`, which takes a second parameter for the type of the mutable store.

Make sure to test this on your local system to ensure that your connections all work as expected before deploying to the server, which you can do with `perseus deploy` and by then copying the `pkg/` directory to the server.

This approach may seem more resilient and modern, but it comes with a severe downside: speed. Every request that involves mutable data (so any request for a revalidating page or an incrementally generated one) must go through four trips (an extra one to and from the database) rather than two, which is twice as many as usual! This will bring down your site's time to first byte (TTFB) radically, so you should ensure that your mutable store is as close to your server as possible so that the latency between them is negligible. If this performance pitfall is not acceptable, you should use an old-school hosting provider instead.
