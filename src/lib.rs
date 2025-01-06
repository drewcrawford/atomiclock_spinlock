/*!
A simple spinlock.

![logo](art/logo.png)

This is a simple spinlock. It is not a fair lock, and it does not provide any way to sleep the current thread if the lock is not available.

 */

use core::ops::{Deref, DerefMut};
use logwise::interval::PerfwarnInterval;

/**
A simple spinlock type.
 */
#[derive(Debug)]
pub struct Lock<T> {
    lock: atomiclock::AtomicLock<T>
}

/**
A guard that provides access to the data in the lock.
 */
#[derive(Debug)]
#[must_use]
pub struct Guard<'a, T>(atomiclock::Guard<'a, T>);

impl <'a, T> Guard<'a, T> {
    pub fn get_mut(&mut self) -> &mut T {
        self.0.as_mut()
    }
}

//drop - we forward to the atomiclock implementation, duh

impl<T> Lock<T> {
    /**
    Creates a new lock.
*/
    pub const fn new(data: T) -> Lock<T> {
        Lock {
            lock: atomiclock::AtomicLock::new(data)
        }
    }

    /**
    Spins until the lock can be acquired.
*/
    pub fn spin_lock(&self) -> Guard<'_,T> {
        loop {
            match self.lock.lock() {
                None => {}
                Some(guard) => {return Guard(guard)}
            }

        }
    }

    /**
    Spins until the lock can be acquired, issuing a perfwarn if spinning were needed due to contention.

    This function is appropriate to use when:
    1.  A spinlock is correct and easy to write.
    2.  You have the suspicion there's a "better" lock-free algorithm, but the tradeoffs are unclear. Worse cache coherency, more code, etc.
    3.  It would be nice to collect some data that would actually drive the decision to write a lock-free algorithm, but to do that you first have to write a program.
    */
    pub fn spin_lock_warn(&self) -> Guard<'_, T> {
        let mut _warn: Option<PerfwarnInterval>;
        loop {
            match self.lock.lock() {
                None => {
                    _warn = Some(logwise::perfwarn_begin!("spin_lock_warn is spinning; investigate ways to reduce contention"));
                }
                Some(guard) => {
                    _warn = None;
                    return Guard(guard);
                }
            }
        }
    }

    /**
    Spins until the lock is available, or times out.
*/
    pub fn spin_lock_until(&self, deadline: std::time::Instant) -> Option<Guard<'_,T>> {
        loop {
            if std::time::Instant::now() > deadline {
                return None;
            }
            match self.lock.lock() {
                None => {}
                Some(guard) => {return Some(Guard(guard))}
            }

        }
    }

    /**
    No spin; provides access to the lock if available.
*/
    pub fn try_lock(&self) -> Option<Guard<'_,T>> {
        match self.lock.lock() {
            None => None,
            Some(guard) => Some(Guard(guard))
        }
    }

    /**
    Consumes the lock and returns the inner data.
*/
    pub fn into_inner(self) -> T {
        self.lock.into_inner()
    }


    /**
    Unsafely provides access to the underlying data.

    # Safety
    This function is unsafe because it allows access to the data without a lock.
*/
    pub unsafe fn data(&self) -> &mut T {
        self.lock.data()
    }
}

/*boilerplate
Locks are not clone, so not copy, Eq, Ord, Hash, etc.
Can pass-through default for default type
can support From for the data type

 */
impl<T: Default> Default for Lock<T> {
    fn default() -> Lock<T> {
        Lock::new(Default::default())
    }
}
impl<T> From<T> for Lock<T> {
    fn from(data: T) -> Lock<T> {
        Lock::new(data)
    }
}

/*
Guard is not copy/clone, so not eq, ord, hash, etc.
No default for guard
I guess it could be created via from
 */

impl<'a, T> From<&'a Lock<T>> for Guard<'a,T> {
    /**
    Implements From by spinning until the lock is acquired.
    */
    fn from(lock: &'a Lock<T>) -> Guard<'a,T> {
        lock.spin_lock()
    }
}

impl<T> AsRef<T> for Guard<'_,T> {
    fn as_ref(&self) -> &T {
        self.0.as_ref()
    }
}

impl<T> AsMut<T> for Guard<'_,T> {
    fn as_mut(&mut self) -> &mut T {
        self.0.as_mut()
    }
}

impl<T> Deref for Guard<'_,T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0.deref()
    }
}

impl<T> DerefMut for Guard<'_,T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0.deref_mut()
    }
}


