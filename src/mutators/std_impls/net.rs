use super::*;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

/// The default mutator for `Ipv4Addr` values.
///
/// See the [`ipv4_addr()`] function to create new instances and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct Ipv4AddrMutator {
    _private: (),
}

/// Create a new mutator for `Ipv4Addr` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use std::net::Ipv4Addr;
///
/// let mut value = Ipv4Addr::LOCALHOST;
///
/// let mut mutator = m::ipv4_addr();
///
/// let mut session = Session::new();
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {value}");
/// }
///
/// // Example output:
/// //
/// //     value = 0.0.0.0
/// //     value = 192.168.0.1
/// //     value = 255.255.255.255
/// //     value = 10.0.0.1
/// //     value = 10.0.0.229
/// # Ok(()) }
/// # foo().unwrap();
/// ```
pub fn ipv4_addr() -> Ipv4AddrMutator {
    Ipv4AddrMutator { _private: () }
}

impl Mutate<Ipv4Addr> for Ipv4AddrMutator {
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut Ipv4Addr) -> Result<()> {
        // Mutate an octet.
        for i in 0..4 {
            c.mutation(|ctx| {
                let octets = value.octets();
                let mut new = octets;
                new[i] = ctx.rng().gen_u8();
                *value = Ipv4Addr::from(new);
                Ok(())
            })?;
        }

        // Special: loopback.
        c.mutation(|_ctx| {
            *value = Ipv4Addr::new(127, 0, 0, 1);
            Ok(())
        })?;

        // Special: unspecified.
        c.mutation(|_ctx| {
            *value = Ipv4Addr::new(0, 0, 0, 0);
            Ok(())
        })?;

        // Special: broadcast.
        c.mutation(|_ctx| {
            *value = Ipv4Addr::new(255, 255, 255, 255);
            Ok(())
        })?;

        // Special: private 192.168.0.1.
        c.mutation(|_ctx| {
            *value = Ipv4Addr::new(192, 168, 0, 1);
            Ok(())
        })?;

        // Special: private 10.0.0.1.
        c.mutation(|_ctx| {
            *value = Ipv4Addr::new(10, 0, 0, 1);
            Ok(())
        })?;

        Ok(())
    }
}

impl Generate<Ipv4Addr> for Ipv4AddrMutator {
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<Ipv4Addr> {
        let a = ctx.rng().gen_u8();
        let b = ctx.rng().gen_u8();
        let c = ctx.rng().gen_u8();
        let d = ctx.rng().gen_u8();
        Ok(Ipv4Addr::new(a, b, c, d))
    }
}

impl DefaultMutate for Ipv4Addr {
    type DefaultMutate = Ipv4AddrMutator;
}

/// The default mutator for `Ipv6Addr` values.
///
/// See the [`ipv6_addr()`] function to create new instances and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct Ipv6AddrMutator {
    _private: (),
}

/// Create a new mutator for `Ipv6Addr` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use std::net::Ipv6Addr;
///
/// let mut value = Ipv6Addr::LOCALHOST;
///
/// let mut mutator = m::ipv6_addr();
///
/// let mut session = Session::new();
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {value}");
/// }
///
/// // Example output:
/// //
/// //     value = ::f886:0:1
/// //     value = ff02::1
/// //     value = ff02::cb11:1
/// //     value = ff02::372c:0:cb11:1
/// //     value = ff02:0:2a39:0:372c:0:cb11:1
/// # Ok(()) }
/// # foo().unwrap();
/// ```
pub fn ipv6_addr() -> Ipv6AddrMutator {
    Ipv6AddrMutator { _private: () }
}

impl Mutate<Ipv6Addr> for Ipv6AddrMutator {
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut Ipv6Addr) -> Result<()> {
        // Mutate a segment.
        for i in 0..8 {
            c.mutation(|ctx| {
                let segs = value.segments();
                let mut new = segs;
                new[i] = ctx.rng().gen_u16();
                *value = Ipv6Addr::from(new);
                Ok(())
            })?;
        }

        // Special: loopback (::1).
        c.mutation(|_ctx| {
            *value = Ipv6Addr::LOCALHOST;
            Ok(())
        })?;

        // Special: unspecified (::).
        c.mutation(|_ctx| {
            *value = Ipv6Addr::UNSPECIFIED;
            Ok(())
        })?;

        // Special: all-nodes multicast (ff02::1).
        c.mutation(|_ctx| {
            *value = Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 1);
            Ok(())
        })?;

        Ok(())
    }
}

impl Generate<Ipv6Addr> for Ipv6AddrMutator {
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<Ipv6Addr> {
        let segs: [u16; 8] = [
            ctx.rng().gen_u16(),
            ctx.rng().gen_u16(),
            ctx.rng().gen_u16(),
            ctx.rng().gen_u16(),
            ctx.rng().gen_u16(),
            ctx.rng().gen_u16(),
            ctx.rng().gen_u16(),
            ctx.rng().gen_u16(),
        ];
        Ok(Ipv6Addr::from(segs))
    }
}

impl DefaultMutate for Ipv6Addr {
    type DefaultMutate = Ipv6AddrMutator;
}

/// The default mutator for `IpAddr` values.
///
/// See the [`ip_addr()`] function to create new instances and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct IpAddrMutator {
    v4: Ipv4AddrMutator,
    v6: Ipv6AddrMutator,
}

/// Create a new mutator for `IpAddr` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use std::net::IpAddr;
///
/// let mut value: IpAddr = "127.0.0.1".parse().unwrap();
///
/// let mut mutator = m::ip_addr();
///
/// let mut session = Session::new();
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {value}");
/// }
///
/// // Example output:
/// //
/// //     value = 127.0.0.41
/// //     value = 186.0.0.41
/// //     value = 186.0.0.252
/// //     value = 10.0.0.1
/// //     value = 10.0.117.1
/// # Ok(()) }
/// # foo().unwrap();
/// ```
pub fn ip_addr() -> IpAddrMutator {
    IpAddrMutator::default()
}

impl Mutate<IpAddr> for IpAddrMutator {
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut IpAddr) -> Result<()> {
        // Mutate the inner address.
        match value {
            IpAddr::V4(ref mut addr) => self.v4.mutate(c, addr)?,
            IpAddr::V6(ref mut addr) => self.v6.mutate(c, addr)?,
        }

        // Switch between V4 and V6.
        c.mutation(|ctx| {
            *value = match *value {
                IpAddr::V4(_) => IpAddr::V6(self.v6.generate(ctx)?),
                IpAddr::V6(_) => IpAddr::V4(self.v4.generate(ctx)?),
            };
            Ok(())
        })?;

        Ok(())
    }
}

impl Generate<IpAddr> for IpAddrMutator {
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<IpAddr> {
        if ctx.rng().gen_bool() {
            Ok(IpAddr::V4(self.v4.generate(ctx)?))
        } else {
            Ok(IpAddr::V6(self.v6.generate(ctx)?))
        }
    }
}

impl DefaultMutate for IpAddr {
    type DefaultMutate = IpAddrMutator;
}

/// The default mutator for `SocketAddrV4` values.
///
/// See the [`socket_addr_v4()`] function to create new instances and for
/// example usage.
#[derive(Clone, Debug, Default)]
pub struct SocketAddrV4Mutator {
    addr: Ipv4AddrMutator,
}

/// Create a new mutator for `SocketAddrV4` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use std::net::SocketAddrV4;
///
/// let mut value: SocketAddrV4 = "127.0.0.1:8080".parse().unwrap();
///
/// let mut mutator = m::socket_addr_v4();
///
/// let mut session = Session::new();
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {value}");
/// }
///
/// // Example output:
/// //
/// //     value = 127.0.0.1:51985
/// //     value = 172.0.0.1:51985
/// //     value = 172.0.40.1:51985
/// //     value = 172.0.40.1:22969
/// //     value = 172.0.14.1:22969
/// # Ok(()) }
/// # foo().unwrap();
/// ```
pub fn socket_addr_v4() -> SocketAddrV4Mutator {
    SocketAddrV4Mutator::default()
}

impl Mutate<SocketAddrV4> for SocketAddrV4Mutator {
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut SocketAddrV4) -> Result<()> {
        // Mutate the address.
        c.mutation(|ctx| {
            let mut ip = *value.ip();
            let result = ctx.mutate_with(&mut self.addr, &mut ip);
            value.set_ip(ip);
            result
        })?;

        // Mutate the port.
        c.mutation(|ctx| {
            value.set_port(ctx.rng().gen_u16());
            Ok(())
        })?;

        Ok(())
    }
}

impl Generate<SocketAddrV4> for SocketAddrV4Mutator {
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<SocketAddrV4> {
        let ip = self.addr.generate(ctx)?;
        let port = ctx.rng().gen_u16();
        Ok(SocketAddrV4::new(ip, port))
    }
}

impl DefaultMutate for SocketAddrV4 {
    type DefaultMutate = SocketAddrV4Mutator;
}

/// The default mutator for `SocketAddrV6` values.
///
/// See the [`socket_addr_v6()`] function to create new instances and for
/// example usage.
#[derive(Clone, Debug, Default)]
pub struct SocketAddrV6Mutator {
    addr: Ipv6AddrMutator,
}

/// Create a new mutator for `SocketAddrV6` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use std::net::SocketAddrV6;
///
/// let mut value: SocketAddrV6 = "[::1]:8080".parse().unwrap();
///
/// let mut mutator = m::socket_addr_v6();
///
/// let mut session = Session::new();
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {value}");
/// }
///
/// // Example output:
/// //
/// //     value = [::1]:47657
/// //     value = [0:6b2::1]:47657
/// //     value = [0:6b2::1%3565297535]:47657
/// //     value = [0:6b2::1%3565297535]:30810
/// //     value = [0:6b2::1%75952059]:30810
/// # Ok(()) }
/// # foo().unwrap();
/// ```
pub fn socket_addr_v6() -> SocketAddrV6Mutator {
    SocketAddrV6Mutator::default()
}

impl Mutate<SocketAddrV6> for SocketAddrV6Mutator {
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut SocketAddrV6) -> Result<()> {
        // Mutate the address.
        c.mutation(|ctx| {
            let mut ip = *value.ip();
            let result = ctx.mutate_with(&mut self.addr, &mut ip);
            value.set_ip(ip);
            result
        })?;

        // Mutate the port.
        c.mutation(|ctx| {
            value.set_port(ctx.rng().gen_u16());
            Ok(())
        })?;

        // Mutate the flowinfo.
        c.mutation(|ctx| {
            value.set_flowinfo(ctx.rng().gen_u32());
            Ok(())
        })?;

        // Mutate the scope_id.
        c.mutation(|ctx| {
            value.set_scope_id(ctx.rng().gen_u32());
            Ok(())
        })?;

        Ok(())
    }
}

impl Generate<SocketAddrV6> for SocketAddrV6Mutator {
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<SocketAddrV6> {
        let ip = self.addr.generate(ctx)?;
        let port = ctx.rng().gen_u16();
        let flowinfo = ctx.rng().gen_u32();
        let scope_id = ctx.rng().gen_u32();
        Ok(SocketAddrV6::new(ip, port, flowinfo, scope_id))
    }
}

impl DefaultMutate for SocketAddrV6 {
    type DefaultMutate = SocketAddrV6Mutator;
}

/// The default mutator for `SocketAddr` values.
///
/// See the [`socket_addr()`] function to create new instances and for example
/// usage.
#[derive(Clone, Debug, Default)]
pub struct SocketAddrMutator {
    v4: SocketAddrV4Mutator,
    v6: SocketAddrV6Mutator,
}

/// Create a new mutator for `SocketAddr` values.
///
/// # Example
///
/// ```
/// # fn foo() -> mutatis::Result<()> {
/// use mutatis::{mutators as m, Mutate, Session};
/// use std::net::SocketAddr;
///
/// let mut value: SocketAddr = "127.0.0.1:8080".parse().unwrap();
///
/// let mut mutator = m::socket_addr();
///
/// let mut session = Session::new();
/// for _ in 0..5 {
///     session.mutate_with(&mut mutator, &mut value)?;
///     println!("value = {value}");
/// }
///
/// // Example output:
/// //
/// //     value = 10.0.0.1:8080
/// //     value = [6d8c:85ab:8ef5:7347:c496:407a:a9e8:b67f%2315231880]:20328
/// //     value = [6d8c:85ab:8ef5:7347:c496:407a:a9e8:b67f%2315231880]:20328
/// //     value = [ff02::1%2315231880]:20328
/// //     value = 64.139.39.75:13206
/// # Ok(()) }
/// # foo().unwrap();
/// ```
pub fn socket_addr() -> SocketAddrMutator {
    SocketAddrMutator::default()
}

impl Mutate<SocketAddr> for SocketAddrMutator {
    #[inline]
    fn mutate(&mut self, c: &mut Candidates, value: &mut SocketAddr) -> Result<()> {
        // Mutate the inner address.
        match value {
            SocketAddr::V4(ref mut addr) => self.v4.mutate(c, addr)?,
            SocketAddr::V6(ref mut addr) => self.v6.mutate(c, addr)?,
        }

        // Switch between V4 and V6.
        c.mutation(|ctx| {
            *value = match *value {
                SocketAddr::V4(_) => SocketAddr::V6(self.v6.generate(ctx)?),
                SocketAddr::V6(_) => SocketAddr::V4(self.v4.generate(ctx)?),
            };
            Ok(())
        })?;

        Ok(())
    }
}

impl Generate<SocketAddr> for SocketAddrMutator {
    #[inline]
    fn generate(&mut self, ctx: &mut Context) -> Result<SocketAddr> {
        if ctx.rng().gen_bool() {
            Ok(SocketAddr::V4(self.v4.generate(ctx)?))
        } else {
            Ok(SocketAddr::V6(self.v6.generate(ctx)?))
        }
    }
}

impl DefaultMutate for SocketAddr {
    type DefaultMutate = SocketAddrMutator;
}
