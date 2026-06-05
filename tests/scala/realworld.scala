package com.example.store

import scala.concurrent.Future
import scala.concurrent.ExecutionContext

// A realistic e-commerce domain model

trait Repository[A, ID] {
  def findById(id: ID): Future[Option[A]]
  def findAll(): Future[List[A]]
  def save(entity: A): Future[A]
  def delete(id: ID): Future[Boolean]
}

case class ProductId(value: Long)
case class UserId(value: Long)

case class Product(
  id: ProductId,
  name: String,
  price: BigDecimal,
  stock: Int
)

case class User(
  id: UserId,
  email: String,
  name: String
)

case class OrderLine(product: Product, quantity: Int) {
  def subtotal: BigDecimal = product.price * quantity
}

case class Order(id: Long, user: User, lines: List[OrderLine]) {
  def total: BigDecimal = lines.map(_.subtotal).sum
  def itemCount: Int = lines.map(_.quantity).sum
}

trait ProductRepository extends Repository[Product, ProductId] {
  def findByName(name: String): Future[List[Product]]
  def findInStock(): Future[List[Product]]
  def updateStock(id: ProductId, delta: Int): Future[Boolean]
}

trait OrderRepository extends Repository[Order, Long] {
  def findByUser(userId: UserId): Future[List[Order]]
  def findRecent(limit: Int): Future[List[Order]]
}

class OrderService(
  orders: OrderRepository,
  products: ProductRepository
)(using ec: ExecutionContext) {
  val MaxOrderLines: Int = 50
  val MinimumOrderAmount: BigDecimal = BigDecimal("0.01")

  def placeOrder(user: User, lines: List[OrderLine]): Future[Order] =
    Future.failed(new NotImplementedError)

  def cancelOrder(orderId: Long): Future[Boolean] =
    Future.failed(new NotImplementedError)

  def getOrderSummary(userId: UserId): Future[String] =
    Future.failed(new NotImplementedError)

  private def validateLines(lines: List[OrderLine]): Boolean =
    lines.nonEmpty && lines.size <= MaxOrderLines
}

object OrderService {
  val ServiceVersion: String = "1.0.0"

  def apply(
    orders: OrderRepository,
    products: ProductRepository
  )(using ec: ExecutionContext): OrderService =
    new OrderService(orders, products)
}
