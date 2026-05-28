# Carteira Digital de Investimentos em Rust

Projeto desenvolvido para entrega do bootcamp **Santander 2026 - Rust AI Developer**, no módulo de construção de uma carteira digital de investimentos com Rust.

## Objetivo

A aplicação demonstra uma base full stack em Rust para gerenciar uma carteira de investimentos. O projeto foi estruturado a partir das aulas do módulo, aplicando conceitos de:

- backend HTTP com **Axum**;
- frontend renderizado no servidor com **Askama**;
- rotas de login, cadastro, dashboard e compras;
- autenticação stateless com **JWT** armazenado em cookie HTTP-only;
- API administrativa para cadastro/atualização de ativos;
- cálculo de posição, valor atual e lucro/prejuízo;
- testes unitários para regras de negócio;
- organização em módulos reutilizáveis.

## Funcionalidades implementadas

- Cadastro de usuário.
- Login com usuário de demonstração.
- Cookie de sessão com JWT.
- Dashboard com ativos e posições da carteira.
- Registro de compras de ativos.
- API REST para listar ativos.
- Endpoint administrativo para criar/atualizar ativos usando `x-admin-secret`.
- Regras de negócio testadas em `src/lib.rs`.

## Como executar

Pré-requisito: Rust instalado via `rustup`.

```bash
cargo run
```

Por padrão o servidor usa a porta `8080`. Se quiser escolher outra porta:

```bash
PORT=3001 cargo run
```

Depois acesse:

```text
http://127.0.0.1:8080
```

Usuário de demonstração:

```text
e-mail: rafael@example.com
senha: 123456
```

## API administrativa

Listar ativos:

```bash
curl http://127.0.0.1:8080/api/assets
```

Criar ou atualizar ativo:

```bash
curl -X POST http://127.0.0.1:8080/api/assets \
  -H 'content-type: application/json' \
  -H 'x-admin-secret: admin' \
  -d '{"symbol":"ETH","name":"Ethereum","current_price_cents":1800000}'
```

## Testes

```bash
cargo test
```

## Relação com as aulas

O projeto consolida os principais tópicos vistos no curso:

1. criação de servidor web com Axum;
2. organização de estado da aplicação;
3. rotas de API para administração de ativos;
4. uso de templates server-side com Askama;
5. autenticação stateless com JWT e cookies;
6. tela de usuário para visualizar ativos e registrar compras;
7. testes das regras centrais da carteira.

A versão atual usa armazenamento em memória para facilitar a execução local e a avaliação do repositório. A estrutura foi mantida de forma que a camada de persistência possa ser substituída por SQLx/PostgreSQL, como apresentado nas aulas.

## Descrição para entrega na DIO

Projeto desenvolvido como parte do bootcamp Santander 2026 - Rust AI Developer, no módulo de construção de uma carteira digital de investimentos com Rust.

A aplicação implementa uma estrutura full stack utilizando Rust no backend e frontend renderizado pelo servidor. O projeto explora criação de APIs com Axum, templates HTML com Askama, autenticação stateless com JWT/cookies e gerenciamento de ativos financeiros do usuário.

Durante o desenvolvimento foram praticados conceitos de programação assíncrona, organização de rotas, tratamento de erros, autenticação, modelagem de dados e integração entre backend e frontend em Rust.

O objetivo principal do projeto é demonstrar o uso de Rust para desenvolvimento web completo, aplicando segurança de tipos, organização de código e bibliotecas modernas do ecossistema Rust.
