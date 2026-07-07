import 'dart:async';
import 'dart:convert';

// ── CASE 1: Class — mixed spacing ─────────────────────────────────────────
class User {
  final int id;
  final String  name;
  final String email;
  final DateTime? createdAt;

  User({ required this.id , required this.name , required this.email , this.createdAt });

  factory User.fromJson(Map<String,dynamic> json) {
    return User(
      id:json['id'] as int,
      name:json['name'] as String,
      email:json['email'] as String,
    );
  }

  Map<String,dynamic> toJson()=>{
    'id':id,
    'name':name,
    'email':email,
  };

  @override
  String toString() => 'User($id, $name, $email)';
}

// ── CASE 2: Abstract class and mixin ──────────────────────────────────────
abstract class Repository<T> {
  Future<T?> findById(int id);
  Future<List<T>> findAll();
  Future<void> save(T entity);
  Future<void> delete(int id);
}

mixin Validatable {
  bool isValid();
  List<String> get validationErrors;
}

// ── CASE 3: Extension methods ─────────────────────────────────────────────
extension StringExtensions on String {
  bool get isValidEmail =>
      RegExp(r'^[a-zA-Z0-9.]+@[a-zA-Z0-9]+\.[a-zA-Z]+$').hasMatch(this);

  String toTitleCase() {
    if (isEmpty) return this;
    return split(' ').map((w)=>w[0].toUpperCase()+w.substring(1)).join(' ');
  }
}

// ── CASE 4: Async/await and streams ──────────────────────────────────────
Future<List<User>> fetchUsers(String baseUrl) async {
  final response = await Future.delayed(const Duration(milliseconds:100));
  return [];
}

Stream<int> countStream(int max) async* {
  for (var i=0; i<=max; i++) {
    await Future.delayed(const Duration(milliseconds:100));
    yield i;
  }
}

// ── CASE 5: Long line ─────────────────────────────────────────────────────
Future<void> veryLongFunctionNameThatExceedsLineWidth(String parameterOne, int parameterTwo, double parameterThree, {bool parameterFour = false}) async {
  print('$parameterOne $parameterTwo $parameterThree $parameterFour');
}

// ── CASE 6: Trailing whitespace ───────────────────────────────────────────
void trailingExample() {   
  final x = 1;   
  final y = 2;  
  print(x + y);
}
