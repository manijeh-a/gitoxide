use crate::revision;
#[cfg(feature = "revision")]
use crate::{bstr::BStr, Id};

/// Methods for resolving revisions by spec or working with the commit graph.
impl crate::Repository {
    /// Parse a revision specification and turn it into the object(s) it describes, similar to `git rev-parse`.
    ///
    /// # Deviation
    ///
    /// - `@` actually stands for `HEAD`, whereas `git` resolves it to the object pointed to by `HEAD` without making the
    ///   `HEAD` ref available for lookups.
    #[doc(alias = "revparse", alias = "git2")]
    #[cfg(feature = "revision")]
    pub fn rev_parse<'a>(&self, spec: impl Into<&'a BStr>) -> Result<revision::Spec<'_>, revision::spec::parse::Error> {
        revision::Spec::from_bstr(
            spec,
            self,
            revision::spec::parse::Options {
                object_kind_hint: self.config.object_kind_hint,
                ..Default::default()
            },
        )
    }

    /// Parse a revision specification and return single object id as represented by this instance.
    #[doc(alias = "revparse_single", alias = "git2")]
    #[cfg(feature = "revision")]
    pub fn rev_parse_single<'repo, 'a>(
        &'repo self,
        spec: impl Into<&'a BStr>,
    ) -> Result<Id<'repo>, revision::spec::parse::single::Error> {
        let spec = spec.into();
        self.rev_parse(spec)?
            .single()
            .ok_or(revision::spec::parse::single::Error::RangedRev { spec: spec.into() })
    }

    /// Obtain the best merge-base between commit `one` and `two`, or fail if there is none.
    ///
    /// # Performance
    /// For repeated calls, prefer [`merge_base_with_cache()`](crate::Repository::merge_base_with_cache()).
    /// Also be sure to [set an object cache](crate::Repository::object_cache_size_if_unset) to accelerate repeated commit lookups.
    #[cfg(feature = "revision")]
    pub fn merge_base(
        &self,
        one: impl Into<gix_hash::ObjectId>,
        two: impl Into<gix_hash::ObjectId>,
    ) -> Result<Id<'_>, super::merge_base::Error> {
        use crate::prelude::ObjectIdExt;
        let one = one.into();
        let two = two.into();
        let cache = self.commit_graph_if_enabled()?;
        let mut graph = self.revision_graph(cache.as_ref());
        let bases = gix_revision::merge_base(one, &[two], &mut graph)?.ok_or(super::merge_base::Error::NotFound {
            first: one,
            second: two,
        })?;
        Ok(bases[0].attach(self))
    }

    /// Obtain the best merge-base between commit `one` and `two`, or fail if there is none, providing a
    /// commit-graph `cache` to potentially greatly accelerate the operation.
    ///
    /// # Performance
    /// Be sure to [set an object cache](crate::Repository::object_cache_size_if_unset) to accelerate repeated commit lookups.
    #[cfg(feature = "revision")]
    pub fn merge_base_with_cache(
        &self,
        one: impl Into<gix_hash::ObjectId>,
        two: impl Into<gix_hash::ObjectId>,
        cache: Option<&gix_commitgraph::Graph>,
    ) -> Result<Id<'_>, super::merge_base_with_cache::Error> {
        use crate::prelude::ObjectIdExt;
        let one = one.into();
        let two = two.into();
        let mut graph = self.revision_graph(cache);
        let bases = gix_revision::merge_base(one, &[two], &mut graph)?.ok_or(
            super::merge_base_with_cache::Error::NotFound {
                first: one,
                second: two,
            },
        )?;
        Ok(bases[0].attach(self))
    }

    /// Obtain all merge-bases between commit `one` and `others`, or an empty list if there is none, providing a
    /// commit-graph `cache` to potentially greatly accelerate the operation.
    ///
    /// # Performance
    /// Be sure to [set an object cache](crate::Repository::object_cache_size_if_unset) to accelerate repeated commit lookups.
    #[doc(alias = "merge_bases_many", alias = "git2")]
    #[cfg(feature = "revision")]
    pub fn merge_bases_many_with_cache(
        &self,
        one: impl Into<gix_hash::ObjectId>,
        others: &[gix_hash::ObjectId],
        cache: Option<&gix_commitgraph::Graph>,
    ) -> Result<Vec<Id<'_>>, gix_revision::merge_base::Error> {
        use crate::prelude::ObjectIdExt;
        let one = one.into();
        let mut graph = self.revision_graph(cache);
        Ok(gix_revision::merge_base(one, others, &mut graph)?
            .unwrap_or_default()
            .into_iter()
            .map(|id| id.attach(self))
            .collect())
    }

    /// Create the baseline for a revision walk by initializing it with the `tips` to start iterating on.
    ///
    /// It can be configured further before starting the actual walk.
    #[doc(alias = "revwalk", alias = "git2")]
    pub fn rev_walk(
        &self,
        tips: impl IntoIterator<Item = impl Into<gix_hash::ObjectId>>,
    ) -> revision::walk::Platform<'_> {
        revision::walk::Platform::new(tips, self)
    }
}
