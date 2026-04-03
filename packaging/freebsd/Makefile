PORTNAME=	flume
DISTVERSION=	1.2.4
CATEGORIES=	irc

MAINTAINER=	emilio@flumeirc.io
COMMENT=	Modern terminal IRC client with scripting and LLM support
WWW=		https://github.com/FlumeIRC/flume

LICENSE=	APACHE20
LICENSE_FILE=	${WRKSRC}/LICENSE

USES=		cargo
USE_GITHUB=	yes
GH_ACCOUNT=	FlumeIRC

CARGO_BUILD_ARGS=	-p flume-tui
CARGO_INSTALL_PATH=	flume-tui

post-install:
	${INSTALL_MAN} ${WRKSRC}/doc/flume.1 ${STAGEDIR}${PREFIX}/man/man1

.include <bsd.port.mk>
